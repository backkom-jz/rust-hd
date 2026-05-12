use chrono::NaiveDateTime;
use sqlx::{MySqlPool, Row};

use crate::models::{
    CreateGame8848Score, Game8848ScoreListResponse, Game8848ScoreQuery, Game8848ScoreRow,
};

#[derive(Clone)]
pub struct Game8848Service {
    pool: MySqlPool,
}

impl Game8848Service {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_score(&self, body: CreateGame8848Score) -> Result<(), String> {
        let nickname = body.nickname.trim().to_string();
        if nickname.is_empty() {
            return Err("昵称不能为空".into());
        }
        if nickname.chars().count() > 16 {
            return Err("昵称最多 16 个字符".into());
        }
        if body.height < 0.0 {
            return Err("高度不能为负数".into());
        }
        if body.height > 10_000.0 {
            return Err("高度超过允许范围".into());
        }
        if body.duration_sec > 3_600 {
            return Err("游戏时长超过允许范围".into());
        }

        sqlx::query(
            r#"INSERT INTO game8848_scores (nickname, height, duration_sec, reached_8848, created_at)
               VALUES (?, ?, ?, ?, NOW())"#,
        )
        .bind(nickname)
        .bind(body.height)
        .bind(body.duration_sec as i64)
        .bind(body.reached_8848 as i8)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("game8848_scores insert: {e}");
            "数据库写入失败".to_string()
        })?;

        Ok(())
    }

    pub async fn top_scores(
        &self,
        query: &Game8848ScoreQuery,
    ) -> Result<Game8848ScoreListResponse, String> {
        let limit = query.normalized_limit() as i64;
        let rows = sqlx::query(
            r#"SELECT nickname, height, duration_sec, reached_8848, created_at
               FROM game8848_scores
               ORDER BY height DESC, id ASC
               LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("game8848_scores query: {e}");
            "数据库查询失败".to_string()
        })?;

        let scores: Vec<Game8848ScoreRow> = rows
            .into_iter()
            .map(|row| {
                let created_at: NaiveDateTime = row.get("created_at");
                let reached_8848: i8 = row.get("reached_8848");
                Game8848ScoreRow {
                    nickname: row.get("nickname"),
                    height: row.get("height"),
                    duration_sec: row.get::<i64, _>("duration_sec") as u32,
                    reached_8848: reached_8848 != 0,
                    created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            })
            .collect();

        let total = scores.len();
        Ok(Game8848ScoreListResponse { total, scores })
    }
}
