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

---

## 生产配置实例（服务器 `47.116.213.182`，已放行 `8088`）

推荐：**Nginx 监听 80（及后续 443）**，Actix 只绑定 **`127.0.0.1:8088`**。这样公网可关 `8088`，只开 `80`/`443`，由 Nginx 统一入口。

### A. 服务器目录与二进制

```bash
sudo mkdir -p /opt/actix-hd/static
# 将本仓库的 target/release/actix-hd 上传到服务器，或在该机 cargo build --release
sudo install -m 755 actix-hd /opt/actix-hd/
sudo cp -r src/static/* /opt/actix-hd/static/
```

### B. 环境变量 `/opt/actix-hd/.env`

参考仓库内 [`deploy/production.env.example`](deploy/production.env.example)，至少包含：

```env
BIND_ADDR=127.0.0.1:8088
STATIC_DIR=/opt/actix-hd/static
DATABASE_URL=mysql://...
```

```bash
sudo chmod 600 /opt/actix-hd/.env
```

### C. systemd

```bash
sudo cp deploy/actix-hd.service /etc/systemd/system/actix-hd.service
sudo systemctl daemon-reload
sudo systemctl enable --now actix-hd
sudo systemctl status actix-hd
```

确认本机可访问：`curl -sS http://127.0.0.1:8088/checkin | head`

### D. Nginx

```bash
sudo cp deploy/nginx-actix-hd.conf /etc/nginx/sites-available/actix-hd
sudo ln -sf /etc/nginx/sites-available/actix-hd /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

浏览器访问：**`http://47.116.213.182/checkin`**（走 80 端口，不再写 `:8088`）。

### E. 云安全组 / 防火墙

| 方案 | 需放行（入站） |
|------|----------------|
| **推荐（Nginx 对外）** | **TCP 80**（及 **443** 若上 HTTPS）；Actix 仅本机，**可关闭公网 8088** |
| 不用 Nginx、直连 Actix | **TCP 8088**，且 `.env` 中 **`BIND_ADDR=0.0.0.0:8088`** |

### F. 可选：仅直连 8088（无 Nginx）

`.env` 使用 `BIND_ADDR=0.0.0.0:8088`，安全组放行 8088，访问 **`http://47.116.213.182:8088/checkin`**。静态资源仍须配置 **`STATIC_DIR`**。

### G. MySQL（RDS）

应用跑在 `47.116.213.182` 上时，RDS 安全组需允许 **该 ECS 的内网 IP**（或公网 IP，若走公网连接）访问 **3306**。连接串建议优先 **内网地址**，延迟更低、不计公网流量费。
