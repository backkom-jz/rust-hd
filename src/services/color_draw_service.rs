use chrono::NaiveDateTime;
use sqlx::{MySqlPool, Row};

use crate::models::{
    ColorDrawBatchView, ColorDrawMemberView, ColorDrawResultResponse, ColorDrawSessionResponse,
    CreateColorDrawBatch, COLOR_POOL,
};

#[derive(Clone)]
pub struct ColorDrawService {
    pool: MySqlPool,
}

impl ColorDrawService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn current_session(&self) -> Result<ColorDrawSessionResponse, String> {
        if let Some(id) = self.find_open_batch_id().await? {
            let batch = self.load_batch(id).await?;
            return Ok(ColorDrawSessionResponse { batch: Some(batch) });
        }

        if let Some(id) = self.find_latest_locked_batch_id().await? {
            let batch = self.load_batch(id).await?;
            return Ok(ColorDrawSessionResponse { batch: Some(batch) });
        }

        Ok(ColorDrawSessionResponse { batch: None })
    }

    pub async fn create_batch(
        &self,
        body: CreateColorDrawBatch,
    ) -> Result<ColorDrawBatchView, String> {
        let names = normalize_names(body.names)?;

        if self.find_open_batch_id().await?.is_some() {
            return Err("当前仍有进行中的批次，请先清空再开新一批".into());
        }

        let mut tx = self.pool.begin().await.map_err(|e| {
            eprintln!("color_draw begin: {e}");
            "数据库事务失败".to_string()
        })?;

        let res = sqlx::query(
            r#"INSERT INTO color_draw_batches (status, created_at)
               VALUES ('open', NOW())"#,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("color_draw_batches insert: {e}");
            "数据库写入失败".to_string()
        })?;

        let batch_id = res.last_insert_id();

        for (i, name) in names.iter().enumerate() {
            sqlx::query(
                r#"INSERT INTO color_draw_members (batch_id, person_name, sort_order)
                   VALUES (?, ?, ?)"#,
            )
            .bind(batch_id)
            .bind(name)
            .bind(i as u8)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("color_draw_members insert: {e}");
                "数据库写入失败".to_string()
            })?;
        }

        tx.commit().await.map_err(|e| {
            eprintln!("color_draw commit: {e}");
            "数据库提交失败".to_string()
        })?;

        self.load_batch(batch_id).await
    }

    pub async fn draw_next(&self) -> Result<ColorDrawResultResponse, String> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            eprintln!("color_draw begin: {e}");
            "数据库事务失败".to_string()
        })?;

        let batch_id: u64 = sqlx::query_scalar(
            r#"SELECT id FROM color_draw_batches
               WHERE status = 'open'
               ORDER BY id DESC
               LIMIT 1
               FOR UPDATE"#,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("color_draw lock batch: {e}");
            "数据库查询失败".to_string()
        })?
        .ok_or_else(|| "没有进行中的批次".to_string())?;

        let next = sqlx::query(
            r#"SELECT id, person_name FROM color_draw_members
               WHERE batch_id = ? AND color IS NULL
               ORDER BY sort_order ASC
               LIMIT 1
               FOR UPDATE"#,
        )
        .bind(batch_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("color_draw next member: {e}");
            "数据库查询失败".to_string()
        })?
        .ok_or_else(|| "本批次已抽完".to_string())?;

        let member_id: u64 = next.get("id");
        let person_name: String = next.get("person_name");

        let used_rows = sqlx::query(
            r#"SELECT color FROM color_draw_members
               WHERE batch_id = ? AND color IS NOT NULL"#,
        )
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("color_draw used colors: {e}");
            "数据库查询失败".to_string()
        })?;

        let used: Vec<String> = used_rows
            .into_iter()
            .map(|r| r.get::<String, _>("color"))
            .collect();

        let remaining: Vec<&str> = COLOR_POOL
            .iter()
            .copied()
            .filter(|c| !used.iter().any(|u| u == c))
            .collect();

        if remaining.is_empty() {
            return Err("没有剩余颜色".into());
        }

        // MySQL FLOOR(RAND()*n) 返回 DOUBLE，用应用侧取模避免 sqlx 类型解码失败。
        let pick_idx = {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            ((nanos as usize)
                .wrapping_mul(31)
                .wrapping_add(member_id as usize)
                .wrapping_add(batch_id as usize))
                % remaining.len()
        };
        let color = remaining[pick_idx].to_string();

        sqlx::query(
            r#"UPDATE color_draw_members
               SET color = ?, drawn_at = NOW()
               WHERE id = ?"#,
        )
        .bind(&color)
        .bind(member_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            eprintln!("color_draw assign: {e}");
            "数据库写入失败".to_string()
        })?;

        let remaining_after = remaining.len() - 1;
        if remaining_after == 0 {
            sqlx::query(
                r#"UPDATE color_draw_batches
                   SET status = 'locked', locked_at = NOW()
                   WHERE id = ?"#,
            )
            .bind(batch_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                eprintln!("color_draw lock: {e}");
                "数据库写入失败".to_string()
            })?;
        }

        tx.commit().await.map_err(|e| {
            eprintln!("color_draw commit: {e}");
            "数据库提交失败".to_string()
        })?;

        let batch = self.load_batch(batch_id).await?;
        Ok(ColorDrawResultResponse {
            person_name,
            color,
            batch,
        })
    }

    /// 放弃当前进行中的批次（未抽完也锁定入库），以便开下一批。
    pub async fn abandon_open(&self) -> Result<(), String> {
        let Some(batch_id) = self.find_open_batch_id().await? else {
            return Ok(());
        };

        sqlx::query(
            r#"UPDATE color_draw_batches
               SET status = 'locked', locked_at = NOW()
               WHERE id = ? AND status = 'open'"#,
        )
        .bind(batch_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("color_draw abandon: {e}");
            "数据库写入失败".to_string()
        })?;

        Ok(())
    }

    async fn find_open_batch_id(&self) -> Result<Option<u64>, String> {
        sqlx::query_scalar(
            r#"SELECT id FROM color_draw_batches
               WHERE status = 'open'
               ORDER BY id DESC
               LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("color_draw open: {e}");
            "数据库查询失败".to_string()
        })
    }

    async fn find_latest_locked_batch_id(&self) -> Result<Option<u64>, String> {
        sqlx::query_scalar(
            r#"SELECT id FROM color_draw_batches
               WHERE status = 'locked'
               ORDER BY id DESC
               LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("color_draw latest locked: {e}");
            "数据库查询失败".to_string()
        })
    }

    async fn load_batch(&self, batch_id: u64) -> Result<ColorDrawBatchView, String> {
        let batch_row = sqlx::query(
            r#"SELECT id, status, created_at, locked_at
               FROM color_draw_batches WHERE id = ?"#,
        )
        .bind(batch_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("color_draw load batch: {e}");
            "数据库查询失败".to_string()
        })?
        .ok_or_else(|| "批次不存在".to_string())?;

        let member_rows = sqlx::query(
            r#"SELECT person_name, sort_order, color, drawn_at
               FROM color_draw_members
               WHERE batch_id = ?
               ORDER BY sort_order ASC"#,
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("color_draw load members: {e}");
            "数据库查询失败".to_string()
        })?;

        let members: Vec<ColorDrawMemberView> = member_rows
            .into_iter()
            .map(|row| {
                let drawn_at: Option<NaiveDateTime> = row.try_get("drawn_at").ok().flatten();
                ColorDrawMemberView {
                    person_name: row.get("person_name"),
                    sort_order: row.get("sort_order"),
                    color: row.try_get("color").ok().flatten(),
                    drawn_at: drawn_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
                }
            })
            .collect();

        let used: Vec<&str> = members
            .iter()
            .filter_map(|m| m.color.as_deref())
            .collect();
        let remaining_colors: Vec<String> = COLOR_POOL
            .iter()
            .copied()
            .filter(|c| !used.contains(c))
            .map(str::to_string)
            .collect();
        let next_person = members
            .iter()
            .find(|m| m.color.is_none())
            .map(|m| m.person_name.clone());

        let created_at: NaiveDateTime = batch_row.get("created_at");
        let locked_at: Option<NaiveDateTime> = batch_row.try_get("locked_at").ok().flatten();

        Ok(ColorDrawBatchView {
            id: batch_row.get("id"),
            status: batch_row.get("status"),
            created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            locked_at: locked_at.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
            members,
            remaining_colors,
            next_person,
        })
    }
}

fn normalize_names(raw: Vec<String>) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for name in raw {
        let n = name.trim().to_string();
        if n.is_empty() {
            return Err("名字不能为空".into());
        }
        if n.chars().count() > 16 {
            return Err("每个名字最多 16 个字符".into());
        }
        if names.iter().any(|x: &String| x == &n) {
            return Err(format!("名字重复：{n}"));
        }
        names.push(n);
    }
    if names.len() != 6 {
        return Err("必须恰好录入 6 个名字".into());
    }
    Ok(names)
}
