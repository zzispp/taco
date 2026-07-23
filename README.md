# Taco

[English](README.en.md)

Taco 是一个使用 Rust/Axum 与 Next.js 构建的管理后台应用。后端遵循 DDD 和 Clean Architecture，前端遵循 Feature-Sliced Design（FSD）。生产构建会将静态导出的前端嵌入 `taco` 可执行文件；开发时前端以独立 Next.js 进程运行。

## 项目概览

- PostgreSQL、Redis、SQLx migration 与类型化 API
- 用户、RBAC、系统管理、调度、审计、可观测性、验证码与文件管理
- 严格 YAML 启动配置：配置文件由 `taco --config <path>` 显式指定
- 管理员权限完全由 RBAC 角色和菜单绑定决定，不存在身份标记绕过
- 支持简体中文、英文和繁体中文的界面与 API 错误响应

## 目录与架构

`apps/backend` 仅是组合根：负责启动、依赖装配、路由与 migration 命令；不得承载领域业务规则。

后端 bounded context：

- `crates/audit`、`crates/observability`、`crates/user`、`crates/rbac`、`crates/system`、`crates/scheduler`、`crates/captcha` 与 `crates/file`。
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

`src/app/**/page.tsx` 只负责路由入口、元数据和守卫；页面组合属于 `pages-layer`。

## 贡献约定

- 领域规则只能放在拥有它的 bounded context；通用 crate、DTO、HTTP handler 和组合根不能吸收业务规则。
- 启动基础设施配置只来自通过 `--config` 传入的 YAML；可在线调整的业务/运行参数只保存在 `sys_config`。一个语义只能有一个有效来源。
- 未发布且可重建开发数据库的 migration baseline 可按明确决策破坏性调整；已部署或需保留数据的 schema 变更必须新增前向 migration。migration 与种子数据必须提供有效默认值。
- UI 文案放入既有 i18n namespace，不能在组件中硬编码；URL locale 与后端 wire locale 映射只从 `locale-contract.json` 派生。
- 提交前运行 Rust 质量门禁；完整的架构、配置、国际化与测试规则以 [AGENTS.md](AGENTS.md) 为准。

## 启动配置

`config/config.example.yaml` 定义完整配置结构，实际运行配置使用不提交的 `config/config.yaml`：

```bash
mkdir -p config
cp config/config.example.yaml config/config.yaml
```

将示例中的每个 `<...>` 占位符替换为真实值。配置加载严格执行下列规则：

- 每个字段都必须显式提供；未知字段、缺失字段、重复 `--config`、空值和未替换的 `<...>` 占位符都会使启动失败。
- YAML 不支持环境变量插值，也没有隐式默认值。可选 Redis 字段也必须显式写为值或 `null`。
- `data_directory` 可以是绝对路径或相对路径；相对路径以 YAML 文件所在目录解析，加载后运行时只接收绝对路径。仓库模板的 `../local-data` 会解析为项目根目录的 `./local-data`。Local File Provider 固定使用 `<data_directory>/files`，并在其中维护 `objects/`、`parts/` 和 `derivatives/`；不需要也不能配置第二个本地存储根目录。
- YAML 承载 `server`、`data_directory`、`database`、`jwt`、`redis`、`user.online_sessions`、`http`、`metrics`、`audit`、`client_info` 与 `scheduler`。修改 YAML 后必须重启 Taco，运行时不会热加载这些值。

使用以下命令生成 `jwt.secret`：

```bash
cargo run -p backend --bin taco -- secret generate-jwt
```

该命令不读取或修改 YAML。将唯一输出完整复制到 `config/config.yaml` 的 `jwt.secret`，不要把密钥提交到仓库或作为命令行参数传递。

启动服务始终显式传入配置路径：

```bash
taco --config <CONFIG_PATH>
```

仓库本地开发的等价命令为：

```bash
cargo run -p backend --bin taco -- --config config/config.yaml
```

## Migration 与初始化数据

`database.auto_migrate` 是必填布尔值：

- 设为 `true` 时，服务会在接受请求前应用前向 migration 并校验 schema。
- 设为 `false` 时，服务只校验 schema；存在待应用、脏状态或校验和不匹配的 migration 会拒绝启动。生产环境建议设为 `false`，由显式运维步骤执行迁移。

Schema 运维子命令为 `migration status` 和 `migration up`：

```bash
taco --config <CONFIG_PATH> migration status
taco --config <CONFIG_PATH> migration up
```

未发布且可重建数据库的开发 baseline 可破坏性调整；调整后必须重建开发数据库并重新应用全部 migration。已部署或需保留数据的实例，每次 schema 变更都必须新增前向 migration。应用 migration 后必须重启 Taco，使进程以已验证的新 schema 重新建立运行时依赖。管理员初始化数据只创建系统 `admin` 角色和显式菜单绑定，不创建用户。

首次部署，或恢复一个不存在启用内置 `admin`（`system=true`）角色用户的实例时，在服务启动前显式创建管理员：

```bash
taco --config <path> administrator bootstrap --username <username> --email <email> --password-stdin
```

`--password-stdin` 只读取标准输入的第一行密码，不接受密码命令行参数，也不会把密码写入 YAML 或命令输出。

该命令仅在数据库中不存在启用内置 `admin` 角色用户时成功，并在数据库事务中创建用户并绑定该角色。服务启动时不会自动创建或恢复管理员；若不存在该管理员，服务会明确失败。管理员用户、角色绑定和数据权限始终由数据库中的 RBAC 管理。

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

创建本地 YAML 配置并替换所有占位符。模板中的 `data_directory: ../local-data` 已解析到项目根目录的 `./local-data`，无需手动填写数据目录。默认开发 Compose 使用 PostgreSQL `127.0.0.1:5435` 和 Redis `127.0.0.1:6381`；将 `database.password` 填为与 Compose 相同的本地密码，并使用上述命令生成独立、真实且至少 32 UTF-8 字节的 `jwt.secret`。`TACO_DATABASE_PASSWORD` 只用于启动开发 PostgreSQL 容器，不是 Taco 运行时配置：

```bash
mkdir -p config
cp config/config.example.yaml config/config.yaml
export TACO_DATABASE_PASSWORD='<LOCAL_POSTGRESQL_PASSWORD>'
just services-up
```

Rust 集成测试从本地 `config/config.yaml` 读取 PostgreSQL 管理连接。每个用例都会创建、连接并销毁独立的临时数据库，不会在 `database.name` 指向的数据库中执行 migration 或业务表写入；配置中的 PostgreSQL 用户必须能够创建数据库、终止连接并删除数据库。

示例配置默认 `database.auto_migrate: false`。在第一个终端显式应用 migration、创建首个管理员并启动后端：

```bash
just backend-migration up
cargo run -p backend --bin taco -- --config config/config.yaml administrator bootstrap --username <username> --email <email> --password-stdin
just run-backend
```

在第二个终端启动独立前端：

```bash
pnpm dev:frontend
```

前端位于 `http://localhost:8082`，会将同源 `/api/*` 转发到 `http://localhost:3000`。开发后端不嵌入静态前端；后端不在默认地址时，可为 Next.js 进程设置仅服务端可见的 `TACO_DEV_BACKEND_URL`。

停止本地依赖：

```bash
just services-down
```

## 生产交付

构建发布二进制：

```bash
just build-release
```

生产 Compose 只运行 Taco；PostgreSQL 与 Redis 是外部、由运维管理的依赖。将生产 YAML 放置在宿主机 `/etc/taco/config.yaml`，保留模板中的 `data_directory: ../local-data`。Compose 将配置挂载到容器 `/app/config/config.yaml`，因此相对路径会解析为 `/app/local-data`，并由命名 `taco-data` volume 持久化。生产环境建议将 `database.auto_migrate` 设为 `false`：新数据库必须在首次启动前显式执行 migration 并创建管理员；已运行实例升级时执行 migration 后重启服务。

在编辑生产 YAML 的 `jwt.secret` 前，构建镜像并生成密钥：

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco secret generate-jwt
```

将输出复制到宿主机 `/etc/taco/config.yaml` 的 `jwt.secret`，不要提交该文件或通过命令参数传递密钥。

Compose 仅发布 `127.0.0.1:3000`。浏览器和 `/api` 必须使用同一公开 origin；代理必须清除客户端伪造的转发头，并写入规范的 `X-Forwarded-For`、`X-Forwarded-Host`、`X-Forwarded-Proto`。不要从公网暴露 `/metrics`、`/docs` 或 `/openapi.json`。

完整的 Docker、反向代理、升级和配置变更流程见 [生产 Docker 部署](deployment.cn.md)。英文版本见 [Production Docker Deployment](deployment.md)。

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

`/health` 是存活探针。`/ready` 在 HTTP 服务成功启动后返回 `200`；配置、schema 和依赖初始化在监听端口前完成，因此它不是持续依赖健康探针。
