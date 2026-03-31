use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::{MySqlPool, Row};

/// 与围栏、坐标校验等并列的业务错误文案；供 handler 映射 HTTP 状态码。
pub const ERR_CHECKIN_ALREADY_TODAY: &str = "今日已签到，同一手机号每天只能签到一次";

use crate::geo::Fence;
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

    /// 当前围栏：**优先**读取表 `geo_fence_config`（`singleton='x'`）；失败或无行时使用环境变量 [`Fence::from_env`]。改库后**无需重启**即可在下一请求生效。
    pub async fn get_fence(&self) -> Fence {
        Self::resolve_fence(&self.pool).await
    }

    async fn resolve_fence(pool: &MySqlPool) -> Fence {
        match Self::fetch_fence_from_db(pool).await {
            Ok(Some(f)) => f,
            Ok(None) => Fence::from_env(),
            Err(e) => {
                eprintln!("geo_fence_config read failed, using env fallback: {e}");
                Fence::from_env()
            }
        }
    }

    async fn fetch_fence_from_db(pool: &MySqlPool) -> Result<Option<Fence>, sqlx::Error> {
        let row = sqlx::query(
            r#"SELECT center_lat, center_lng, radius_km, venue_name
               FROM geo_fence_config WHERE singleton = 'x'"#,
        )
        .fetch_optional(pool)
        .await?;

        Ok(row.map(|r| {
            let mut venue: String = r.get("venue_name");
            venue = venue.trim().to_string();
            if venue.is_empty() {
                venue = crate::geo::DEFAULT_VENUE_NAME.to_string();
            }
            Fence {
                center_lat: r.get("center_lat"),
                center_lng: r.get("center_lng"),
                radius_km: r.get("radius_km"),
                venue_name: venue,
            }
        }))
    }

    /// 校验地理围栏后写入：页面 `/checkin`，接口 `POST /api/checkins`。
    pub async fn sign_in(&self, body: SignInBody) -> Result<CheckInRecord, String> {
        self.sign_in_inner(body, true).await
    }

    /// 不校验与围栏距离，仍写入坐标及距参考中心的公里数（仅展示）；页面 `/checkin2`，接口 `POST /api/checkins/open`。
    pub async fn sign_in_open(&self, body: SignInBody) -> Result<CheckInRecord, String> {
        self.sign_in_inner(body, false).await
    }

    async fn sign_in_inner(&self, body: SignInBody, enforce_fence: bool) -> Result<CheckInRecord, String> {
        let fence = self.get_fence().await;

        let phone = body.phone.trim().to_string();
        let username = body.username.trim().to_string();
        if phone.is_empty() {
            return Err("手机号不能为空".into());
        }
        if username.is_empty() {
            return Err("用户名不能为空".into());
        }

        let lat = body.latitude;
        let lng = body.longitude;
        if !lat.is_finite() || !lng.is_finite() {
            return Err("请提供有效的定位坐标".into());
        }
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            return Err("请提供有效的定位坐标".into());
        }

        let dist = fence.distance_from_center_km(lat, lng);
        if enforce_fence && !fence.contains(lat, lng) {
            return Err(format!(
                "签到地点需在「{}」周边{}公里内（当前距该中心约{:.2}公里）",
                fence.venue_name,
                fmt_radius(fence.radius_km),
                dist
            ));
        }

        let signed_at_naive = Local::now().naive_local();
        let today = signed_at_naive.date();

        let cnt: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM check_ins WHERE phone = ? AND DATE(signed_at) = ?"#,
        )
        .bind(&phone)
        .bind(today)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("check_ins daily duplicate check: {e}");
            "数据库查询失败".to_string()
        })?;

        if cnt > 0 {
            return Err(ERR_CHECKIN_ALREADY_TODAY.into());
        }

        let res = sqlx::query(
            r#"INSERT INTO check_ins (phone, username, signed_at, latitude, longitude, distance_km)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&phone)
        .bind(&username)
        .bind(signed_at_naive)
        .bind(lat)
        .bind(lng)
        .bind(dist)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("check_ins insert: {e}");
            "数据库写入失败".to_string()
        })?;

        let id = res.last_insert_id();
        let signed_at = naive_as_local(signed_at_naive);

        Ok(CheckInRecord {
            id,
            phone,
            username,
            signed_at,
            distance_km: dist,
        })
    }

    pub async fn stats_for_query(
        &self,
        q: &CheckInDateQuery,
    ) -> Result<CheckInStatsResponse, String> {
        let date = q.to_naive_date().ok_or_else(|| "无效的日期".to_string())?;

        let rows = sqlx::query(
            r#"SELECT phone, username, signed_at, distance_km FROM check_ins
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
                let dist: Option<f64> = row.get("distance_km");
                let distance_km = dist
                    .map(|d| format!("{:.2}", d))
                    .unwrap_or_else(|| "—".to_string());
                CheckInSignerRow {
                    phone,
                    username,
                    signed_at: dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                    distance_km,
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

fn fmt_radius(km: f64) -> String {
    if km.fract().abs() < f64::EPSILON {
        format!("{}", km as i64)
    } else {
        format!("{:.1}", km)
    }
}
