use chrono::NaiveDateTime;
use sqlx::{MySqlPool, Row};

use crate::models::{CreateDinoScore, DinoScoreListResponse, DinoScoreQuery, DinoScoreRow};

#[derive(Clone)]
pub struct DinoService {
    pool: MySqlPool,
}

impl DinoService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_score(&self, body: CreateDinoScore) -> Result<(), String> {
        let nickname = body.nickname.trim().to_string();
        if nickname.is_empty() {
            return Err("昵称不能为空".into());
        }
        if nickname.chars().count() > 16 {
            return Err("昵称最多 16 个字符".into());
        }
        if body.score < 0 {
            return Err("分数不能小于 0".into());
        }
        if body.score > 1_000_000_000 {
            return Err("分数超过允许范围".into());
        }

        sqlx::query(
            r#"INSERT INTO dino_scores (nickname, score, created_at)
               VALUES (?, ?, NOW())"#,
        )
        .bind(nickname)
        .bind(body.score)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("dino_scores insert: {e}");
            "数据库写入失败".to_string()
        })?;

        Ok(())
    }

    pub async fn top_scores(&self, query: &DinoScoreQuery) -> Result<DinoScoreListResponse, String> {
        let limit = query.normalized_limit();
        let sql = format!(
            r#"SELECT nickname, score, created_at
               FROM dino_scores
               ORDER BY score DESC, id ASC
               LIMIT {limit}"#
        );

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await.map_err(|e| {
            eprintln!("dino_scores query: {e}");
            "数据库查询失败".to_string()
        })?;

        let scores: Vec<DinoScoreRow> = rows
            .into_iter()
            .map(|row| {
                let created_at: NaiveDateTime = row.get("created_at");
                DinoScoreRow {
                    nickname: row.get("nickname"),
                    score: row.get("score"),
                    created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            })
            .collect();

        let total = scores.len();
        Ok(DinoScoreListResponse { total, scores })
    }
}
