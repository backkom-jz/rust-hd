use serde::{Deserialize, Serialize};

pub const COLOR_POOL: [&str; 6] = ["白", "灰", "深蓝", "黑", "粉", "蓝"];

#[derive(Debug, Deserialize)]
pub struct CreateColorDrawBatch {
    pub names: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ColorDrawMemberView {
    pub person_name: String,
    pub sort_order: u8,
    pub color: Option<String>,
    pub drawn_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ColorDrawBatchView {
    pub id: u64,
    pub status: String,
    pub created_at: String,
    pub locked_at: Option<String>,
    pub members: Vec<ColorDrawMemberView>,
    pub remaining_colors: Vec<String>,
    pub next_person: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ColorDrawSessionResponse {
    pub batch: Option<ColorDrawBatchView>,
}

#[derive(Debug, Serialize)]
pub struct ColorDrawResultResponse {
    pub person_name: String,
    pub color: String,
    pub batch: ColorDrawBatchView,
}
