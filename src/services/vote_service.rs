use chrono::NaiveDateTime;
use sqlx::{MySqlPool, Row};

use crate::models::{
    CreateVotePoll, SubmitVote, VoteCurrentResponse, VoteHistoryResponse, VoteOptionResult,
    VoteOptionView, VotePhoneQuery, VotePollPublic, VotePollResults, VotePollSummary,
    VoteResultsQuery, VoteStatusResponse,
};

#[derive(Clone)]
pub struct VoteService {
    pool: MySqlPool,
}

impl VoteService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn current_public(&self) -> Result<VoteCurrentResponse, String> {
        let Some(id) = self.find_open_poll_id().await? else {
            return Ok(VoteCurrentResponse { poll: None });
        };
        Ok(VoteCurrentResponse {
            poll: Some(self.load_poll_public(id).await?),
        })
    }

    pub async fn vote_status(&self, query: &VotePhoneQuery) -> Result<VoteStatusResponse, String> {
        let phone = normalize_phone(&query.phone)?;
        let Some(poll_id) = self.find_open_poll_id().await? else {
            return Ok(VoteStatusResponse {
                poll_id: None,
                voted: false,
                option_id: None,
            });
        };
        let option_id = self.find_ballot_option(poll_id, &phone).await?;
        Ok(VoteStatusResponse {
            poll_id: Some(poll_id),
            voted: option_id.is_some(),
            option_id,
        })
    }

    pub async fn submit(&self, body: SubmitVote) -> Result<VotePollResults, String> {
        let phone = normalize_phone(&body.phone)?;

        let mut tx = self.pool.begin().await.map_err(|e| {
            eprintln!("vote begin: {e}");
            "数据库事务失败".to_string()
        })?;

        let poll_id: u64 = sqlx::query_scalar(
            r#"SELECT id FROM vote_polls
               WHERE status = 'open'
               ORDER BY id DESC
               LIMIT 1
               FOR UPDATE"#,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("vote lock poll: {e}");
            "数据库查询失败".to_string()
        })?
        .ok_or_else(|| "当前没有进行中的投票".to_string())?;

        let option_ok: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM vote_options
               WHERE id = ? AND poll_id = ?"#,
        )
        .bind(body.option_id)
        .bind(poll_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("vote option check: {e}");
            "数据库查询失败".to_string()
        })?;
        if option_ok == 0 {
            return Err("选项无效".into());
        }

        let already: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM vote_ballots
               WHERE poll_id = ? AND phone = ?"#,
        )
        .bind(poll_id)
        .bind(&phone)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("vote already: {e}");
            "数据库查询失败".to_string()
        })?;
        if already > 0 {
            return Err("该手机号本场已投票".into());
        }

        sqlx::query(
            r#"INSERT INTO vote_ballots (poll_id, phone, option_id, voted_at)
               VALUES (?, ?, ?, NOW())"#,
        )
        .bind(poll_id)
        .bind(&phone)
        .bind(body.option_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("vote insert: {e}");
            if e.to_string().contains("Duplicate") {
                "该手机号本场已投票".to_string()
            } else {
                "数据库写入失败".to_string()
            }
        })?;

        tx.commit().await.map_err(|e| {
            eprintln!("vote commit: {e}");
            "数据库提交失败".to_string()
        })?;

        self.load_results(poll_id, Some(body.option_id)).await
    }

    pub async fn user_results(&self, query: &VoteResultsQuery) -> Result<VotePollResults, String> {
        let phone = normalize_phone(query.phone.as_deref().unwrap_or(""))?;
        let poll_id = match query.poll_id {
            Some(id) => id,
            None => self
                .find_open_poll_id()
                .await?
                .ok_or_else(|| "当前没有进行中的投票".to_string())?,
        };

        let my_option = self.find_ballot_option(poll_id, &phone).await?;
        if my_option.is_none() {
            return Err("请先完成本场投票后再查看结果".into());
        }

        self.load_results(poll_id, my_option).await
    }

    pub async fn admin_results(&self, poll_id: Option<u64>) -> Result<VotePollResults, String> {
        let id = match poll_id {
            Some(id) => id,
            None => self
                .find_open_poll_id()
                .await?
                .or(self.find_latest_poll_id().await?)
                .ok_or_else(|| "暂无投票场次".to_string())?,
        };
        self.load_results(id, None).await
    }

    pub async fn history(&self) -> Result<VoteHistoryResponse, String> {
        let rows = sqlx::query(
            r#"SELECT p.id, p.title, p.status, p.created_at, p.closed_at,
                      (SELECT COUNT(*) FROM vote_ballots b WHERE b.poll_id = p.id) AS total_votes
               FROM vote_polls p
               ORDER BY p.id DESC
               LIMIT 30"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote history: {e}");
            "数据库查询失败".to_string()
        })?;

        let polls = rows
            .into_iter()
            .map(|row| {
                let created_at: NaiveDateTime = row.get("created_at");
                let closed_at: Option<NaiveDateTime> = row.try_get("closed_at").ok().flatten();
                let total: i64 = row.get("total_votes");
                VotePollSummary {
                    id: row.get("id"),
                    title: row.get("title"),
                    status: row.get("status"),
                    created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    closed_at: closed_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                    total_votes: total.max(0) as u64,
                }
            })
            .collect();

        Ok(VoteHistoryResponse { polls })
    }

    pub async fn create_poll(&self, body: CreateVotePoll) -> Result<VotePollPublic, String> {
        let title = body.title.trim().to_string();
        if title.is_empty() {
            return Err("标题不能为空".into());
        }
        if title.chars().count() > 80 {
            return Err("标题最多 80 个字符".into());
        }

        let mut options = Vec::new();
        for raw in body.options {
            let label = raw.trim().to_string();
            if label.is_empty() {
                continue;
            }
            if label.chars().count() > 80 {
                return Err("选项最多 80 个字符".into());
            }
            if options.iter().any(|x: &String| x == &label) {
                return Err(format!("选项重复：{label}"));
            }
            options.push(label);
        }
        if options.len() < 2 {
            return Err("至少需要 2 个有效选项".into());
        }
        if options.len() > 20 {
            return Err("选项最多 20 个".into());
        }

        if self.find_open_poll_id().await?.is_some() {
            return Err("当前仍有进行中的投票，请先结束本场".into());
        }

        let mut tx = self.pool.begin().await.map_err(|e| {
            eprintln!("vote begin: {e}");
            "数据库事务失败".to_string()
        })?;

        let res = sqlx::query(
            r#"INSERT INTO vote_polls (title, status, created_at)
               VALUES (?, 'open', NOW())"#,
        )
        .bind(&title)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("vote_polls insert: {e}");
            "数据库写入失败".to_string()
        })?;
        let poll_id = res.last_insert_id();

        for (i, label) in options.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO vote_options (poll_id, label, sort_order)
                   VALUES (?, ?, ?)"#,
            )
            .bind(poll_id)
            .bind(label)
            .bind(i as u8)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("vote_options insert: {e}");
                "数据库写入失败".to_string()
            })?;
        }

        tx.commit().await.map_err(|e| {
            eprintln!("vote commit: {e}");
            "数据库提交失败".to_string()
        })?;

        self.load_poll_public(poll_id).await
    }

    pub async fn close_current(&self) -> Result<VotePollResults, String> {
        let poll_id = self
            .find_open_poll_id()
            .await?
            .ok_or_else(|| "当前没有进行中的投票".to_string())?;

        sqlx::query(
            r#"UPDATE vote_polls
               SET status = 'closed', closed_at = NOW()
               WHERE id = ? AND status = 'open'"#,
        )
        .bind(poll_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote close: {e}");
            "数据库写入失败".to_string()
        })?;

        self.load_results(poll_id, None).await
    }

    async fn find_open_poll_id(&self) -> Result<Option<u64>, String> {
        sqlx::query_scalar(
            r#"SELECT id FROM vote_polls
               WHERE status = 'open'
               ORDER BY id DESC
               LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote open: {e}");
            "数据库查询失败".to_string()
        })
    }

    async fn find_latest_poll_id(&self) -> Result<Option<u64>, String> {
        sqlx::query_scalar(
            r#"SELECT id FROM vote_polls
               ORDER BY id DESC
               LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote latest: {e}");
            "数据库查询失败".to_string()
        })
    }

    async fn find_ballot_option(&self, poll_id: u64, phone: &str) -> Result<Option<u64>, String> {
        sqlx::query_scalar(
            r#"SELECT option_id FROM vote_ballots
               WHERE poll_id = ? AND phone = ?
               LIMIT 1"#,
        )
        .bind(poll_id)
        .bind(phone)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote ballot: {e}");
            "数据库查询失败".to_string()
        })
    }

    async fn load_poll_public(&self, poll_id: u64) -> Result<VotePollPublic, String> {
        let poll = sqlx::query(
            r#"SELECT id, title, status, created_at, closed_at
               FROM vote_polls WHERE id = ?"#,
        )
        .bind(poll_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote load poll: {e}");
            "数据库查询失败".to_string()
        })?
        .ok_or_else(|| "投票不存在".to_string())?;

        let option_rows = sqlx::query(
            r#"SELECT id, label, sort_order
               FROM vote_options
               WHERE poll_id = ?
               ORDER BY sort_order ASC, id ASC"#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote load options: {e}");
            "数据库查询失败".to_string()
        })?;

        let options = option_rows
            .into_iter()
            .map(|row| VoteOptionView {
                id: row.get("id"),
                label: row.get("label"),
                sort_order: row.get("sort_order"),
            })
            .collect();

        let created_at: NaiveDateTime = poll.get("created_at");
        let closed_at: Option<NaiveDateTime> = poll.try_get("closed_at").ok().flatten();

        Ok(VotePollPublic {
            id: poll.get("id"),
            title: poll.get("title"),
            status: poll.get("status"),
            created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            closed_at: closed_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            options,
        })
    }

    async fn load_results(
        &self,
        poll_id: u64,
        my_option_id: Option<u64>,
    ) -> Result<VotePollResults, String> {
        let poll = self.load_poll_public(poll_id).await?;

        let count_rows = sqlx::query(
            r#"SELECT o.id, o.label, o.sort_order,
                      COUNT(b.id) AS votes
               FROM vote_options o
               LEFT JOIN vote_ballots b ON b.option_id = o.id
               WHERE o.poll_id = ?
               GROUP BY o.id, o.label, o.sort_order
               ORDER BY o.sort_order ASC, o.id ASC"#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("vote counts: {e}");
            "数据库查询失败".to_string()
        })?;

        let options: Vec<VoteOptionResult> = count_rows
            .into_iter()
            .map(|row| {
                let votes: i64 = row.get("votes");
                VoteOptionResult {
                    id: row.get("id"),
                    label: row.get("label"),
                    sort_order: row.get("sort_order"),
                    votes: votes.max(0) as u64,
                }
            })
            .collect();

        let total_votes = options.iter().map(|o| o.votes).sum();

        Ok(VotePollResults {
            id: poll.id,
            title: poll.title,
            status: poll.status,
            created_at: poll.created_at,
            closed_at: poll.closed_at,
            total_votes,
            options,
            my_option_id,
        })
    }
}

fn normalize_phone(raw: &str) -> Result<String, String> {
    let phone: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    if phone.is_empty() {
        return Err("手机号不能为空".into());
    }
    if phone.chars().count() > 20 {
        return Err("手机号过长".into());
    }
    if !phone.chars().all(|c| c.is_ascii_digit()) {
        return Err("手机号只能包含数字".into());
    }
    if phone.len() < 6 {
        return Err("手机号过短".into());
    }
    Ok(phone)
}
