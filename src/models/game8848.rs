use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateGame8848Score {
    pub nickname: String,
    pub height: f64,
    pub duration_sec: u32,
    pub reached_8848: bool,
}

#[derive(Debug, Deserialize)]
pub struct Game8848ScoreQuery {
    pub limit: Option<u8>,
}

impl Game8848ScoreQuery {
    pub fn normalized_limit(&self) -> usize {
        let n = self.limit.unwrap_or(10);
        n.clamp(1, 50) as usize
    }
}

#[derive(Debug, Serialize)]
pub struct Game8848ScoreRow {
    pub nickname: String,
    pub height: f64,
    pub duration_sec: u32,
    pub reached_8848: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct Game8848ScoreListResponse {
    pub total: usize,
    pub scores: Vec<Game8848ScoreRow>,
}
