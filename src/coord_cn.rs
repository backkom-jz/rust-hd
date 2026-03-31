//! 国内常用坐标：WGS84（GPS/浏览器 geolocation）→ GCJ-02（国测局、高德/腾讯底图）。

use std::f64::consts::PI;

/// 是否在中国大陆外（外域不做偏移）。
pub fn out_of_china(lat: f64, lng: f64) -> bool {
    lng < 72.004 || lng > 137.8347 || lat < 0.8293 || lat > 55.8271
}

fn transform_lat(lng: f64, lat: f64) -> f64 {
    let mut ret = -100.0 + 2.0 * lng + 3.0 * lat + 0.2 * lat * lat + 0.1 * lng * lat + 0.2 * lng.abs().sqrt();
    ret += (20.0 * (6.0 * lng * PI).sin() + 20.0 * (2.0 * lng * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (lat * PI).sin() + 40.0 * (lat / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (160.0 * (lat / 12.0 * PI).sin() + 320.0 * (lat * PI / 30.0).sin()) * 2.0 / 3.0;
    ret
}

fn transform_lng(lng: f64, lat: f64) -> f64 {
    let mut ret = 300.0 + lng + 2.0 * lat + 0.1 * lng * lng + 0.1 * lng * lat + 0.1 * lng.abs().sqrt();
    ret += (20.0 * (6.0 * lng * PI).sin() + 20.0 * (2.0 * lng * PI).sin()) * 2.0 / 3.0;
    ret += (20.0 * (lng * PI).sin() + 40.0 * (lng / 3.0 * PI).sin()) * 2.0 / 3.0;
    ret += (150.0 * (lng / 12.0 * PI).sin() + 300.0 * (lng / 30.0 * PI).sin()) * 2.0 / 3.0;
    ret
}

/// WGS84 → GCJ-02（火星坐标），供高德 Web 服务 `location=lng,lat` 使用。
pub fn wgs84_to_gcj02(lat: f64, lng: f64) -> (f64, f64) {
    if out_of_china(lat, lng) {
        return (lat, lng);
    }
    let dlat = transform_lat(lng - 105.0, lat - 35.0);
    let dlng = transform_lng(lng - 105.0, lat - 35.0);
    let radlat = lat / 180.0 * PI;
    let magic = 1.0 - 0.00669342162296594323 * radlat.sin() * radlat.sin();
    let sqrtmagic = magic.sqrt();
    let dlat2 = (dlat * 180.0) / ((6378245.0 * (1.0 - 0.00669342162296594323)) / (magic * sqrtmagic) * PI);
    let dlng2 = (dlng * 180.0) / ((6378245.0 / sqrtmagic * radlat.cos()) * PI);
    (lat + dlat2, lng + dlng2)
}
