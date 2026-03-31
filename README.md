# rust-hd

基于 Actix-web 的签到示例：MySQL 持久化、签到页与统计页。

## 本地运行

1. 复制 `env.example` 为 `.env` 并填写数据库连接（见文件内说明）。
2. `cargo run`
3. 浏览器访问 `http://127.0.0.1:8088/checkin`（端口可通过环境变量 `BIND_ADDR` 修改）。

## 生产环境部署

### 1. 在服务器上构建或交叉编译

在 **与目标系统相同或兼容** 的环境执行：

```bash
cargo build --release
```

产物：`target/release/actix-hd`。

也可在 CI 中构建，将二进制与静态目录一并发布到服务器。

### 2. 准备目录与静态资源

HTML 模板已编译进二进制；**背景图等** 来自磁盘目录，生产环境必须可访问：

```bash
sudo mkdir -p /opt/actix-hd/static
sudo cp -r src/static/* /opt/actix-hd/static/
sudo install -m 755 target/release/actix-hd /opt/actix-hd/
```

设置环境变量 **`STATIC_DIR=/opt/actix-hd/static`**（否则程序仍尝试使用开发机上的编译路径，图片会 404）。

### 3. 环境变量（勿把 `.env` 提交到 Git）

在 systemd、`docker run -e` 或主机面板中配置，至少包括：

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` 或 `DB_*` | MySQL 连接（与 `env.example` 一致） |
| `STATIC_DIR` | 静态文件目录绝对路径 |
| `BIND_ADDR` | 监听地址，常见 `127.0.0.1:8088`（仅本机 + Nginx）或 `0.0.0.0:8088`（容器内） |

RDS / 云数据库需在安全组中 **放行应用服务器访问 3306**。

### 4. 使用 systemd 常驻（示例）

`/etc/systemd/system/actix-hd.service`：

```ini
[Unit]
Description=actix-hd check-in service
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/actix-hd
EnvironmentFile=/opt/actix-hd/.env
ExecStart=/opt/actix-hd/actix-hd
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

将生产用变量写入 **`/opt/actix-hd/.env`**（权限 `chmod 600`），然后：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now actix-hd
```

### 5. Nginx 反向代理与 HTTPS（推荐）

对外只暴露 443，由 Nginx 终止 TLS，反代到本机 Actix：

```nginx
server {
    listen 443 ssl http2;
    server_name your.domain.com;
    ssl_certificate     /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8088;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

此时 **`BIND_ADDR=127.0.0.1:8088`** 即可。

### 6. Docker（可选思路）

镜像内复制 `actix-hd` 二进制与 `static/`，`ENV STATIC_DIR=/app/static`，`EXPOSE 8088`，`DATABASE_URL` 指向宿主机或同一 compose 网络中的 MySQL。数据库表会在进程启动时由 `db::init_schema` 自动创建（若账号有建表权限）。

### 7. 运维注意

- 使用 **`RUST_LOG=actix_web=info`**（若后续接入 `tracing`/`env_logger`）便于排查。
- 连接池上限在 `main.rs` 的 `MySqlPoolOptions::max_connections` 可按并发调整。
- 升级流程：停服务 → 替换二进制 → 如有迁移脚本则执行 → 启动服务。
