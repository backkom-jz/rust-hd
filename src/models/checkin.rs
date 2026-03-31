use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CheckInRecord {
    pub id: u64,
    pub phone: String,
    pub username: String,
    pub signed_at: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
pub struct SignInBody {
    pub phone: String,
    pub username: String,
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
}
