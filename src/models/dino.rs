use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateDinoScore {
    pub nickname: String,
    pub score: i64,
}

#[derive(Debug, Deserialize)]
pub struct DinoScoreQuery {
    pub limit: Option<u8>,
}

impl DinoScoreQuery {
    pub fn normalized_limit(&self) -> usize {
        let n = self.limit.unwrap_or(10);
        n.clamp(1, 50) as usize
    }
}

#[derive(Debug, Serialize)]
pub struct DinoScoreRow {
    pub nickname: String,
    pub score: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct DinoScoreListResponse {
    pub total: usize,
    pub scores: Vec<DinoScoreRow>,
}
