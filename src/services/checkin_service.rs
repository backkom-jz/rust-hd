use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::{MySqlPool, Row};

use crate::models::{
    CheckInDateQuery, CheckInRecord, CheckInSignerRow, CheckInStatsResponse, SignInBody,
};

#[derive(Clone)]
pub struct CheckInService {
    pool: MySqlPool,
}

impl CheckInService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn sign_in(&self, body: SignInBody) -> Result<CheckInRecord, String> {
        let phone = body.phone.trim().to_string();
        let username = body.username.trim().to_string();
        if phone.is_empty() {
            return Err("手机号不能为空".into());
        }
        if username.is_empty() {
            return Err("用户名不能为空".into());
        }

        let signed_at = Local::now().naive_local();

        let res = sqlx::query(
            "INSERT INTO check_ins (phone, username, signed_at) VALUES (?, ?, ?)",
        )
        .bind(&phone)
        .bind(&username)
        .bind(signed_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("check_ins insert: {e}");
            "数据库写入失败".to_string()
        })?;

        let id = res.last_insert_id();
        let signed_at = naive_as_local(signed_at);

        Ok(CheckInRecord {
            id,
            phone,
            username,
            signed_at,
        })
    }

    pub async fn stats_for_query(
        &self,
        q: &CheckInDateQuery,
    ) -> Result<CheckInStatsResponse, String> {
        let date = q.to_naive_date().ok_or_else(|| "无效的日期".to_string())?;

        let rows = sqlx::query(
            r#"SELECT phone, username, signed_at FROM check_ins
               WHERE DATE(signed_at) = ? ORDER BY signed_at ASC"#,
        )
        .bind(date)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("check_ins stats: {e}");
            "数据库查询失败".to_string()
        })?;

        let signers: Vec<CheckInSignerRow> = rows
            .into_iter()
            .map(|row| {
                let phone: String = row.get("phone");
                let username: String = row.get("username");
                let naive: NaiveDateTime = row.get("signed_at");
                let dt = naive_as_local(naive);
                CheckInSignerRow {
                    phone,
                    username,
                    signed_at: dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            })
            .collect();

        let count = signers.len();
        Ok(CheckInStatsResponse { count, signers })
    }
}

fn naive_as_local(naive: NaiveDateTime) -> DateTime<Local> {
    match naive.and_local_timezone(Local) {
        chrono::LocalResult::Single(dt) => dt,
        chrono::LocalResult::Ambiguous(earliest, _) => earliest,
        chrono::LocalResult::None => Local::now(),
    }
}
