use std::path::PathBuf;

/// `BIND_ADDR`，默认 `127.0.0.1:8088`。生产环境常设为 `0.0.0.0:8088`（再由 Nginx 反代）。
pub fn bind_addr() -> String {
    std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8088".to_string())
}

/// 静态资源目录（含 `img/`）。生产环境应设置 `STATIC_DIR` 指向部署机上的目录，勿依赖编译机路径。
pub fn static_dir() -> PathBuf {
    std::env::var("STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/static"))
}

/// Prefer `DATABASE_URL` (`mysql://user:pass@host:port/db?charset=utf8mb4`),
/// or set `DB_HOST`, `DB_USER`, `DB_PASSWORD`, `DB_NAME`, and optional `DB_PORT`.
pub fn database_url() -> Result<String, String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        if url.is_empty() {
            return Err("DATABASE_URL is empty".into());
        }
        return Ok(url);
    }

    let host = std::env::var("DB_HOST").map_err(|_| {
        "Missing DATABASE_URL or DB_HOST (set credentials via environment, not in source code)"
            .to_string()
    })?;
    let port = std::env::var("DB_PORT").unwrap_or_else(|_| "3306".into());
    let user = std::env::var("DB_USER")
        .map_err(|_| "DB_USER is required when DATABASE_URL is not set".to_string())?;
    let password = std::env::var("DB_PASSWORD")
        .map_err(|_| "DB_PASSWORD is required when DATABASE_URL is not set".to_string())?;
    let name = std::env::var("DB_NAME")
        .map_err(|_| "DB_NAME is required when DATABASE_URL is not set".to_string())?;

    let u = urlencoding::encode(&user);
    let p = urlencoding::encode(&password);
    Ok(format!(
        "mysql://{u}:{p}@{host}:{port}/{name}?charset=utf8mb4"
    ))
}
