use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReverseQuery {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Deserialize)]
struct NominatimReverse {
    display_name: Option<String>,
}

/// 将 WGS84 坐标转为可读地址（OpenStreetMap Nominatim）。需服务器能访问外网；失败时前端仍可仅凭坐标签到。
pub async fn reverse_geocode(q: web::Query<ReverseQuery>) -> impl Responder {
    let lat = q.lat;
    let lng = q.lng;
    if !lat.is_finite()
        || !lng.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lng)
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "坐标无效"
        }));
    }

    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&accept-language=zh-CN,zh,en",
        lat, lng
    );

    let client = awc::Client::default();
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
                "error": "地址服务暂时不可用（网络或境外接口受限）",
                "display_name": serde_json::Value::Null
            }));
        }
    };

    if !resp.status().is_success() {
        return HttpResponse::BadGateway().json(serde_json::json!({
            "error": "地址服务返回异常",
            "display_name": serde_json::Value::Null
        }));
    }

    let body = match resp.body().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("nominatim body: {e}");
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": "读取地址响应失败",
                "display_name": serde_json::Value::Null
            }));
        }
    };

    let parsed: NominatimReverse = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("nominatim json: {e}");
            return HttpResponse::BadGateway().json(serde_json::json!({
                "error": "解析地址失败",
                "display_name": serde_json::Value::Null
            }));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "display_name": parsed.display_name,
        "lat": lat,
        "lng": lng,
    }))
}
