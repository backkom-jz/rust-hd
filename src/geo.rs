//! WGS84 球面距离（Haversine）与围栏结构。数值默认来自常量，运行时可由数据库 `geo_fence_config` 覆盖（见 `CheckInService::get_fence`）。

/// 默认中心：上海东方体育中心一带（WGS84 约值）。
pub const DEFAULT_CENTER_LAT: f64 = 31.1569;
pub const DEFAULT_CENTER_LNG: f64 = 121.4746;
pub const DEFAULT_RADIUS_KM: f64 = 5.0;
pub const DEFAULT_VENUE_NAME: &str = "上海东方体育中心";

#[derive(Debug, Clone)]
pub struct Fence {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: f64,
    /// 展示名称（数据库 `venue_name` 或环境变量）
    pub venue_name: String,
}

impl Fence {
    /// 环境变量兜底（数据库无记录或查询失败时使用）。
    pub fn from_env() -> Self {
        Self {
            center_lat: parse_env_f64("GEO_FENCE_LAT", DEFAULT_CENTER_LAT),
            center_lng: parse_env_f64("GEO_FENCE_LNG", DEFAULT_CENTER_LNG),
            radius_km: parse_env_f64("GEO_FENCE_RADIUS_KM", DEFAULT_RADIUS_KM),
            venue_name: std::env::var("GEO_FENCE_VENUE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| DEFAULT_VENUE_NAME.to_string()),
        }
    }

    pub fn distance_from_center_km(&self, lat: f64, lng: f64) -> f64 {
        haversine_km(lat, lng, self.center_lat, self.center_lng)
    }

    pub fn contains(&self, lat: f64, lng: f64) -> bool {
        self.distance_from_center_km(lat, lng) <= self.radius_km + 1e-9
    }
}

fn parse_env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_KM: f64 = 6371.0;
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let h = (d_lat / 2.0).sin().powi(2)
        + lat1r.cos() * lat2r.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * h.sqrt().asin();
    EARTH_KM * c
}
