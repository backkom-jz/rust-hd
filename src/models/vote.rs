use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateVotePoll {
    pub title: String,
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitVote {
    pub phone: String,
    pub option_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct VotePhoneQuery {
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteResultsQuery {
    pub phone: Option<String>,
    pub poll_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct VoteOptionView {
    pub id: u64,
    pub label: String,
    pub sort_order: u8,
}

#[derive(Debug, Serialize)]
pub struct VoteOptionResult {
    pub id: u64,
    pub label: String,
    pub sort_order: u8,
    pub votes: u64,
}

#[derive(Debug, Serialize)]
pub struct VotePollPublic {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub options: Vec<VoteOptionView>,
}

#[derive(Debug, Serialize)]
pub struct VotePollResults {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub total_votes: u64,
    pub options: Vec<VoteOptionResult>,
    pub my_option_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct VoteCurrentResponse {
    pub poll: Option<VotePollPublic>,
}

#[derive(Debug, Serialize)]
pub struct VoteStatusResponse {
    pub poll_id: Option<u64>,
    pub voted: bool,
    pub option_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct VotePollSummary {
    pub id: u64,
    pub title: String,
    pub status: String,
    pub created_at: String,
    pub closed_at: Option<String>,
    pub total_votes: u64,
}

#[derive(Debug, Serialize)]
pub struct VoteHistoryResponse {
    pub polls: Vec<VotePollSummary>,
}
