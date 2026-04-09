use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CheckInRecord {
    pub id: u64,
    pub phone: String,
    pub username: String,
    pub signed_at: DateTime<Local>,
    /// 签到位置到围栏中心的直线距离（公里）
    pub distance_km: f64,
}

#[derive(Debug, Deserialize)]
pub struct SignInBody {
    pub phone: String,
    pub username: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize)]
pub struct CheckInDateQuery {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl CheckInDateQuery {
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day)
    }
}

#[derive(Debug, Serialize)]
pub struct CheckInStatsResponse {
    pub count: usize,
    pub signers: Vec<CheckInSignerRow>,
}

#[derive(Debug, Serialize)]
pub struct CheckInSignerRow {
    pub phone: String,
    pub username: String,
    pub signed_at: String,
    /// 展示用，无历史数据时为 "—"
    pub distance_km: String,
}
