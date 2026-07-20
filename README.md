# Taco

[English](README.en.md)

Taco 是一个使用 Rust/Axum 与 Next.js 构建的管理后台应用。后端遵循 DDD 和 Clean Architecture，前端遵循 Feature-Sliced Design（FSD）。生产构建会将静态导出的前端嵌入 `taco` 可执行文件；开发时前端以独立 Next.js 进程运行。

## 项目概览

- PostgreSQL、Redis、SQLx migration 与类型化 API
- 用户、RBAC、系统管理、调度、审计、可观测性、验证码与安装引导
- 浏览器首次访问完成三步安装，连接信息保存在加密的安装状态中
- 唯一的安装所有者以标记授权，不依赖业务角色或权限
- 支持简体中文、英文和繁体中文的界面与 API 错误响应

## 目录与架构

`apps/backend` 仅是组合根：负责启动、依赖装配、路由与 migration 命令；不得承载领域业务规则。

后端 bounded context：

- `crates/audit`、`crates/observability`、`crates/user`、`crates/rbac`、`crates/system`、`crates/scheduler`、`crates/captcha` 与 `crates/installation`。
- 上下文内部按 `domain`、`application`、`infra`、`api` 分层；`apps/backend` 只在已有上下文能力后完成装配。
- `crates/audit_contract` 维护跨上下文审计契约；`crates/client_info`、`crates/config`、`crates/storage`、`crates/types`、`crates/constants`、`crates/kernel`、`crates/tracing` 是共享基础能力；`crates/rbac_macros` 与 `crates/scheduler_macros` 是配套宏 crate。

SQLx migration 位于 `migrations/`。发布构建先生成 `apps/frontend/out`，再通过 `embedded-frontend` feature 嵌入后端二进制。

前端位于 `apps/frontend/src`，依赖方向固定为：

```text
app -> pages-layer/widgets/features/entities/shared
pages-layer -> widgets/features/entities/shared
widgets -> features/entities/shared
features -> entities/shared
entities -> shared
```

`src/app/**/page.tsx` 只负责路由入口、元数据和守卫；页面组合属于 `pages-layer`。安装页面独立于认证与运行时配置 Provider。

## 贡献约定

- 领域规则只能放在拥有它的 bounded context；通用 crate、DTO、HTTP handler 和组合根不能吸收业务规则。
- 启动基础设施参数保存在加密的安装状态；可在线调整的业务/运行参数只保存在 `sys_config`。一个语义只能有一个有效来源。
- 新 schema 变更新增 migration，绝不修改已经应用的 migration；migration 与种子数据必须提供有效默认值。
- UI 文案放入既有 i18n namespace，不能在组件中硬编码；前端语言为 `cn`、`en`、`tw`，后端 wire locale 为 `zh-CN`、`en`、`zh-TW`。
- 提交前运行 Rust 质量门禁；完整的架构、配置、国际化与测试规则以 [AGENTS.md](AGENTS.md) 为准。

## Bootstrap 与安装

### Bootstrap 输入

| 输入           | 来源                                                      | 使用场景              | 含义                                       |
| -------------- | --------------------------------------------------------- | --------------------- | ------------------------------------------ |
| 数据目录       | `--data-dir` 或 `TACO_DATA_DIR`                           | 服务、migration、重置 | 保存 `installation-state.enc` 和上传文件。 |
| 配置加密根密钥 | `--config-encryption-key` 或 `TACO_CONFIG_ENCRYPTION_KEY` | 服务、migration       | 加密安装状态的 32 字节 Base64URL 密钥。    |
| 监听地址       | `--listen` 或 `TACO_LISTEN_ADDR`                          | 服务                  | 可选，默认 `0.0.0.0:3000`。                |

同一输入不能同时通过命令行与环境变量提供。`taco secrets generate` 不读取 bootstrap 输入；`taco installation reset` 只接受数据目录。

生成根密钥：

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
```

命令输出 `TACO_CONFIG_ENCRYPTION_KEY=<value>`。将该值与数据目录一起迁移；丢失它后无法解密既有安装状态，必须通过显式恢复流程重建状态，旧会话将失效。

### 首次安装

安装状态不存在时，后端进入 setup mode：`/health` 返回 `200`，`/ready` 返回 `503`。发布二进制会把根路径 `/` 重定向到 `/cn/`；独立前端开发时请访问 `http://localhost:8082/cn/setup/`。

安装向导依次执行：

1. 填写并测试 PostgreSQL 主机、端口、数据库、用户名、密码和 TLS 选项。
2. 填写并测试 Redis 主机、端口、可选用户名/密码/数据库和 TLS 选项。
3. 创建初始安装所有者。高级区使用后端提供的 HTTP、指标、会话清理、审计 outbox、IP 定位、调度器与 Redis 前缀默认值。

最终提交会再次验证连接，删除选定 PostgreSQL 数据库的 `public` schema，并对选定 Redis 实例执行 `FLUSHALL`；随后执行全部初始 migration、创建安装所有者、原子写入加密状态并请求进程退出以重启。PostgreSQL 数据库和 Redis 实例必须专供该 Taco 安装使用，`FLUSHALL` 会清除实例的所有逻辑数据库，与选中的数据库编号和 key prefix 无关。

安装所有者没有预置业务角色，但通过安装所有者标记绕过业务权限检查。其他管理员必须在用户与 RBAC 管理中显式创建、分配角色与权限。PostgreSQL、Redis、JWT 和高级安装参数在安装后不可在线修改；已有 `sys_config` 参数仍由其管理 API 负责。

### 重置安装

先停止 Taco，再仅删除本地加密安装状态：

```bash
export TACO_DATA_DIR="$PWD/.local/taco-data"
cargo run -p backend --bin taco -- installation reset --confirm-reset
```

该命令不需要旧根密钥，也不会删除数据目录中的上传文件。使用新根密钥重新启动后会进入 setup mode；若所选 PostgreSQL 已包含 Taco schema，安装提交会明确拒绝，绝不会清空 PostgreSQL 或 Redis。

### 迁移与恢复

服务器迁移的完整单元是 `TACO_DATA_DIR`、配置加密根密钥、PostgreSQL、Redis 和上传文件。正常情况下，将它们一并迁移后直接启动，不要重新运行安装向导。

数据库或 Redis 连接地址变更、但加密安装状态仍存在时，创建只包含 `database` 与 `redis` 的 JSON 文件，并显式重新配置。该命令验证迁移完整性、安装所有者和 Redis `PING` 后，才原子替换状态文件，其余不可变配置与 JWT 密钥保持不变：

```bash
cargo run -p backend --bin taco -- \
  --data-dir /var/lib/taco \
  --config-encryption-key "$TACO_CONFIG_ENCRYPTION_KEY" \
  installation reconfigure --connections /secure/taco-connections.json
```

状态文件丢失、但数据库已迁移时，先输出完整 `InstallationProfile` 模板，填入原有不可变配置和新的连接信息，再使用恢复命令而不是 setup。模板中的 JWT 值会被恢复命令替换：

```bash
cargo run -p backend --bin taco -- installation profile template > /secure/taco-installation-profile.json
```

旧根密钥丢失但 `installation-state.enc` 仍在时，先运行 `installation reset --confirm-reset` 删除该状态文件；它不会修改 PostgreSQL、Redis 或上传文件。随后使用新的根密钥执行恢复。

恢复会验证 schema、安装所有者和 Redis，并生成新的 JWT 密钥；所有用户必须重新登录：

```bash
cargo run -p backend --bin taco -- \
  --data-dir /var/lib/taco \
  --config-encryption-key "$TACO_CONFIG_ENCRYPTION_KEY" \
  installation recover --profile /secure/taco-installation-profile.json
```

## 本地开发

### 前置条件

- Rust toolchain（workspace 使用 edition 2024）
- Node.js `>=22.12.0`
- pnpm `10.33.4`
- Docker 与 Docker Compose
- [just](https://github.com/casey/just)

安装前端依赖：

```bash
pnpm install
```

启动本地 PostgreSQL 与 Redis。`TACO_DATABASE_PASSWORD` 仅供开发 Compose 使用，不是 Taco 运行时配置：

```bash
export TACO_DATABASE_PASSWORD='<local PostgreSQL password>'
just services-up
```

在第一个终端生成根密钥、导出 bootstrap 输入并启动后端：

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
export TACO_DATA_DIR="$PWD/.local/taco-data"
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
cargo run -p backend --bin taco --
```

在第二个终端启动独立前端：

```bash
pnpm dev:frontend
```

前端位于 `http://localhost:8082`，会将同源 `/api/*` 转发到 `http://localhost:3000`。访问 `http://localhost:8082/cn/setup/` 完成首次本地安装；前端会根据安装状态在 setup 与正常应用之间跳转。开发后端不嵌入静态前端，因此不要将 `http://localhost:3000/setup` 作为开发入口。后端不在默认地址时，可为 Next.js 进程设置仅服务端可见的 `TACO_DEV_BACKEND_URL`。

本地 Compose 的向导输入：

| 服务       | 主机        | 端口   | TLS  |
| ---------- | ----------- | ------ | ---- |
| PostgreSQL | `localhost` | `5435` | 关闭 |
| Redis      | `localhost` | `6381` | 关闭 |

PostgreSQL 用户名与数据库均为 `postgres`，密码为 `TACO_DATABASE_PASSWORD` 的值；提供的 Redis 服务没有密码。前端只接受继承的进程环境，并会拒绝工作区根目录或 `apps/frontend` 中的 `.env`、`.env.*` 文件。

停止本地服务：

```bash
just services-down
```

## Migration

首次安装自动执行全部 migration。已安装实例升级后，使用同一个数据目录与根密钥检查或应用前向 migration：

```bash
just backend-migration status
just backend-migration up
```

运维 CLI 只公开 `status` 与 `up`，没有回滚或数据库重置命令。正常运行时发现 pending migration、脏 migration 或校验和不匹配会显式拒绝启动。开发新变更时，新增 migration 与必要的种子/测试，不修改已应用的 migration 文件。

## 生产交付

构建发布二进制：

```bash
just build-release
```

生产 Compose 只运行 Taco；PostgreSQL 与 Redis 是外部、由运维管理的依赖。首次启动前导出根密钥：

```bash
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d --build
```

Compose 将 `taco-data` volume 挂载到 `/data`，并仅发布 `127.0.0.1:3000`。在 HTTPS 反向代理后访问该站点完成安装。浏览器和 `/api` 必须使用同一公开 origin；代理必须清除客户端伪造的转发头，并写入规范的 `X-Forwarded-For`、`X-Forwarded-Host`、`X-Forwarded-Proto`。不要从公网暴露 `/metrics`、`/docs` 或 `/openapi.json`。

完整的 Docker、反向代理、升级与重置契约见 [生产 Docker 部署](deployment.cn.md)。英文版本见 [Production Docker Deployment](deployment.md)。

## 常用命令与验证

```bash
# Rust
just format
just lint
just check
just build
just test
just quality-precommit
just quality-complete
just install-git-hooks

# Frontend
pnpm lint:frontend
pnpm build:frontend
pnpm --filter frontend test
pnpm --filter frontend build:embedded
```

`just quality-precommit` 依次执行格式、Clippy、workspace check 与测试；`just quality-complete` 在其基础上执行 `cargo audit` 和 `cargo deny check`。提交前运行前者，完成 Rust 工作前运行后者。

`/health` 是存活探针：setup 与正常模式均返回 `200`。`/ready` 只在已安装的运行时就绪后返回 `200`，setup mode 返回 `503`。
