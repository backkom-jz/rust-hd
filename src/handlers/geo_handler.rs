use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::coord_cn::wgs84_to_gcj02;

#[derive(Debug, Deserialize)]
pub struct ReverseQuery {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Deserialize)]
struct NominatimReverse {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AmapRegeoRoot {
    status: String,
    #[serde(default)]
    info: Option<String>,
    regeocode: Option<AmapRegeoCode>,
}

#[derive(Debug, Deserialize)]
struct AmapRegeoCode {
    #[serde(default)]
    formatted_address: Option<String>,
}

fn validate_coords(lat: f64, lng: f64) -> bool {
    lat.is_finite()
        && lng.is_finite()
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&lng)
}

/// 逆地理：配置了 `AMAP_WEB_KEY` 时走**高德**（国内稳定）；否则走 OpenStreetMap Nominatim（境外或备用）。
pub async fn reverse_geocode(q: web::Query<ReverseQuery>) -> impl Responder {
    let lat = q.lat;
    let lng = q.lng;
    if !validate_coords(lat, lng) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "坐标无效"
        }));
    }

    let client = awc::Client::default();

    if let Ok(ref key) = std::env::var("AMAP_WEB_KEY") {
        let key = key.trim();
        if !key.is_empty() {
            match reverse_amap(&client, lat, lng, key).await {
                Ok(Some(addr)) => {
                    return HttpResponse::Ok().json(serde_json::json!({
                        "display_name": addr,
                        "lat": lat,
                        "lng": lng,
                        "provider": "amap"
                    }));
                }
                Ok(None) => {}
                Err(e) => eprintln!("amap regeo: {e}"),
            }
        }
    }

    reverse_nominatim(&client, lat, lng).await
}

async fn reverse_amap(
    client: &awc::Client,
    wgs_lat: f64,
    wgs_lng: f64,
    key: &str,
) -> Result<Option<String>, String> {
    let (gcj_lat, gcj_lng) = wgs84_to_gcj02(wgs_lat, wgs_lng);
    let loc = format!("{gcj_lng},{gcj_lat}");
    let url = format!(
        "https://restapi.amap.com/v3/geocode/regeo?key={}&location={}&extensions=base&output=json",
        urlencoding::encode(key),
        loc
    );

    let mut resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("amap http {}", resp.status()));
    }

    let body = resp
        .body()
        .await
        .map_err(|e| e.to_string())?;

    let parsed: AmapRegeoRoot = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    if parsed.status != "1" {
        return Err(parsed.info.unwrap_or_else(|| "amap status not 1".into()));
    }

    Ok(parsed
        .regeocode
        .and_then(|r| r.formatted_address)
        .filter(|s| !s.is_empty()))
}

async fn reverse_nominatim(client: &awc::Client, lat: f64, lng: f64) -> HttpResponse {
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&accept-language=zh-CN,zh,en",
        lat, lng
    );

    let mut resp = match client
        .get(url)
        .insert_header((
            "User-Agent",
            "actix-hd-signin/1.0 (https://github.com/backkom-jz/rust-hd)",
        ))
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("nominatim request: {e}");
            return HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": "地址服务暂时不可用（可配置 AMAP_WEB_KEY 使用高德）",
                "display_name": serde_json::Value::Null,
                "provider": "nominatim"
            }));
        }
    };

    if !resp.status().is_success() {
        return HttpResponse::BadGateway().json(serde_json::json!({
            "error": "地址服务返回异常",
            "display_name": serde_json::Value::Null,
            "provider": "nominatim"
        }));
    }

    let body = match resp.body().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("nominatim body: {e}");
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": "读取地址响应失败",
                "display_name": serde_json::Value::Null,
                "provider": "nominatim"
            }));
        }
    };

    let parsed: NominatimReverse = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("nominatim json: {e}");
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": "解析地址失败",
                "display_name": serde_json::Value::Null,
                "provider": "nominatim"
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "display_name": parsed.display_name,
        "lat": lat,
        "lng": lng,
        "provider": "nominatim"
    }))
}
