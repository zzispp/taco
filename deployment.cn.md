# 生产 Docker 部署

Taco 的生产镜像包含静态导出的前端和一个 `taco` 可执行文件。PostgreSQL 与 Redis 是外部依赖，不由生产 Compose 文件创建；其连接信息在首次访问的安装向导中填写。

## 启动

首次启动前生成配置加密根密钥。此命令不需要数据目录或既有安装状态。仅使用 Docker 时，先构建镜像，再执行运维命令：

```bash
docker build --tag taco:local .
docker run --rm taco:local secrets generate
```

使用检出的 Rust 工具链时，可执行等价命令：

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
```

将生成的值导出到启动 Compose 的 shell：

```bash
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d --build
```

服务仅发布到 `127.0.0.1:3000`。通过主机 HTTPS 反向代理访问公开站点并完成三个安装步骤。Taco 将加密安装状态和可变上传文件存储在挂载至 `/data` 的命名 `taco-data` volume 中。

Compose 存活探针访问 `/health`。安装阶段该端点健康，即使已安装运行时尚未就绪、`/ready` 仍不可用。

## 反向代理契约

在主机侧反向代理终止 TLS，并将上游请求代理到 `http://127.0.0.1:3000`。代理必须移除客户端提供的转发头，并写入规范的 `X-Forwarded-For`、`X-Forwarded-Host` 与 `X-Forwarded-Proto`。Taco 接受这些标准头，无需配置受信任代理 CIDR。

浏览器流量与 `/api` 必须使用同一公开 origin。Taco 负责前端安全响应头和同源 API 行为；代理负责域名对应的 TLS 证书、HSTS 和网络策略。

必须让 `/metrics`、`/docs` 与 `/openapi.json` 保持内网可见，不得通过公开虚拟主机路由这些路径。指标抓取与 API 文档访问仅限运维人员或私有监控网络。

## 运维

已安装实例升级时，先使用新镜像应用前向 migration，再启动服务。`run` 创建一次性运维容器，因此不依赖正常服务先变为 ready：

```bash
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml run --rm taco migration up
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d
```

如需有意重置实例，先停止服务、生成并导出新根密钥，再移除加密安装状态：

```bash
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml stop taco
docker run --rm taco:local secrets generate
export TACO_CONFIG_ENCRYPTION_KEY='<new generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml run --rm taco installation reset --confirm-reset
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d
```

重置命令不需要旧根密钥。下一次安装提交重新验证 PostgreSQL 与 Redis 输入后，Taco 只会在全新目标上执行破坏性重置。若选定 PostgreSQL 数据库已包含 Taco schema，安装会在修改 PostgreSQL 或 Redis 前明确拒绝。`FLUSHALL` 会清除选定 Redis 实例的所有逻辑数据库，而不受数据库编号或 key prefix 影响；用于全新安装的 Redis 目标仍必须专供 Taco 使用。

## 服务器迁移与状态恢复

迁移服务器时，应一并迁移命名 `taco-data` volume、`TACO_CONFIG_ENCRYPTION_KEY`、PostgreSQL、Redis 和上传文件。加密状态文件完好时，迁移后的安装可直接启动，不要重复 Web 安装。

仅 PostgreSQL 或 Redis 端点变化时，挂载数据卷，并运行 `installation reconfigure --connections <path>`。其 JSON 文件包含安装配置中的 `database` 与 `redis` 对象。该命令在原子替换加密状态前，会检查迁移后的 schema、安装所有者与 Redis。

状态文件丢失但 Taco 数据库完好时，使用 `installation profile template` 生成完整 `InstallationProfile` JSON 模板，填入原有不可变配置和当前连接信息后，执行 `installation recover --profile <path>`。恢复会验证相同的数据库和 Redis 不变量，写入新的加密状态，并生成新的 JWT 签名密钥。既有浏览器会话会被有意失效，用户必须重新登录。

若旧根密钥丢失但 `installation-state.enc` 仍存在，先执行 `installation reset --confirm-reset` 仅移除该加密状态文件，再使用新根密钥恢复。该操作不会修改 PostgreSQL、Redis 或上传文件。

## 构建契约

仓库发布命令会先导出前端，再启用 Rust 嵌入 feature：

```bash
just build-release
```

本地开发时，独立运行 Next.js 前端，不要启用嵌入式前端 feature。
