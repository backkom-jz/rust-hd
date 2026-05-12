use sqlx::MySqlPool;

pub async fn init_schema(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS check_ins (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
            phone VARCHAR(32) NOT NULL,
            username VARCHAR(128) NOT NULL,
            signed_at DATETIME NOT NULL,
            latitude DOUBLE NULL COMMENT 'WGS84',
            longitude DOUBLE NULL COMMENT 'WGS84',
            distance_km DOUBLE NULL COMMENT '距围栏中心公里',
            KEY idx_check_ins_signed_at (signed_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dino_scores (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
            nickname VARCHAR(64) NOT NULL,
            score BIGINT NOT NULL,
            created_at DATETIME NOT NULL,
            KEY idx_dino_scores_score (score),
            KEY idx_dino_scores_created_at (created_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await?;

    migrate_legacy_check_ins_geo(pool).await?;
    init_geo_fence_config(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS game8848_scores (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
            nickname VARCHAR(32) NOT NULL,
            height DOUBLE NOT NULL COMMENT '攀爬高度（米）',
            duration_sec INT UNSIGNED NOT NULL COMMENT '游戏时长（秒）',
            reached_8848 TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否登顶',
            created_at DATETIME NOT NULL,
            KEY idx_8848_height (height DESC),
            KEY idx_8848_created_at (created_at)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// 单表一行：`singleton = 'x'`，存储围栏中心、半径与场地名称。可在库内 UPDATE，次请求即生效。
async fn init_geo_fence_config(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS geo_fence_config (
            singleton CHAR(1) NOT NULL PRIMARY KEY COMMENT '固定单例键 x',
            center_lat DOUBLE NOT NULL,
            center_lng DOUBLE NOT NULL,
            radius_km DOUBLE NOT NULL,
            venue_name VARCHAR(128) NOT NULL DEFAULT ''
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
        "#,
    )
    .execute(pool)
    .await?;

    let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM geo_fence_config")
        .fetch_one(pool)
        .await?;

    if cnt == 0 {
        sqlx::query(
            r#"
            INSERT INTO geo_fence_config (singleton, center_lat, center_lng, radius_km, venue_name)
            VALUES ('x', ?, ?, ?, ?)
            "#,
        )
        .bind(crate::geo::DEFAULT_CENTER_LAT)
        .bind(crate::geo::DEFAULT_CENTER_LNG)
        .bind(crate::geo::DEFAULT_RADIUS_KM)
        .bind(crate::geo::DEFAULT_VENUE_NAME)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// 旧库仅有 phone/username/signed_at 时补齐地理字段。
async fn migrate_legacy_check_ins_geo(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let cnt: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM information_schema.COLUMNS
        WHERE TABLE_SCHEMA = DATABASE()
          AND TABLE_NAME = 'check_ins'
          AND COLUMN_NAME = 'latitude'
        "#,
    )
    .fetch_one(pool)
    .await?;

    if cnt == 0 {
        sqlx::query(
            r#"
            ALTER TABLE check_ins
                ADD COLUMN latitude DOUBLE NULL COMMENT 'WGS84' AFTER signed_at,
                ADD COLUMN longitude DOUBLE NULL COMMENT 'WGS84' AFTER latitude,
                ADD COLUMN distance_km DOUBLE NULL COMMENT '距围栏中心公里' AFTER longitude
            "#,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
