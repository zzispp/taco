# taco

`taco` is a Rust + Next.js admin monorepo with a DDD + Clean Architecture backend and a Feature-Sliced Design frontend.

The backend is organized around user, RBAC, system, scheduler, captcha, and audit capabilities. The frontend uses FSD layers under `apps/frontend/src` and keeps page composition out of Next.js route entries.

## Highlights

- Rust backend organized by bounded context and Clean Architecture layers
- Next.js 16 frontend organized by FSD layers instead of template-style technical folders
- JWT sign-in, sign-up, refresh, and current-user endpoints
- RBAC for users, roles, API permissions, and menus
- System administration for departments, posts, dictionaries, configuration, notices, and online sessions
- Scheduled jobs plus operation and login audit log management with XLSX exports
- PostgreSQL persistence with SQLx migrations
- PostgreSQL-backed online sessions plus Redis-backed RBAC cache, captcha state, and authentication lock state
- pnpm workspace + Cargo workspace in one repository

## Stack

### Backend

- Rust
- Axum
- SQLx
- PostgreSQL
- Redis
- Utoipa / Scalar

### Frontend

- Next.js 16
- React 19
- TypeScript
- MUI 7
- Axios
- SWR

## Architecture

### Backend: DDD + Clean Architecture

The backend keeps business logic inside domain crates and uses `apps/backend` as the composition root.

- `apps/backend`: app entry, dependency wiring, HTTP bootstrap, migration commands
- `crates/audit`: operation and login audit bounded context with management APIs and exports
- `crates/audit_contract`: cross-context audit event and endpoint contracts
- `crates/client_info`: shared client address, user-agent, and IP location infrastructure
- `crates/user`: user bounded context with `domain`, `application`, `infra`, `api`
- `crates/rbac`: RBAC bounded context with `domain`, `application`, `infra`, `api`
- `crates/system`: system administration bounded context
- `crates/scheduler`: scheduled job bounded context
- `crates/captcha`: captcha application and API capability
- `crates/config`: typed config loading and validation
- `crates/storage`: database and storage foundation
- `crates/types`: shared transport DTOs and request/response types
- `crates/constants`, `crates/kernel`, `crates/tracing`: shared support crates

Backend rule: business rules stay in their owning bounded-context crate. Do not push domain logic into `apps/backend` or generic support crates.

### Frontend: FSD on top of Next.js App Router

The frontend follows Feature-Sliced Design inside `apps/frontend/src`.

```text
apps/frontend/src
├── app          # Next.js App Router entry + FSD app layer
├── pages-layer  # page composition layer; named this way because Next.js reserves src/pages
├── widgets      # page blocks and layout/business containers
├── features     # user actions and use-case level UI behavior
├── entities     # business entities, entity UI, entity queries/types
└── shared       # pure shared infrastructure, UI, config, routes, theme, assets
```

Layer dependency direction is fixed:

- `app -> pages-layer/widgets/features/entities/shared`
- `pages-layer -> widgets/features/entities/shared`
- `widgets -> features/entities/shared`
- `features -> entities/shared`
- `entities -> shared`
- `shared -> shared`

Important frontend rule: `src/app/**/page.tsx` stays thin and delegates page composition to `src/pages-layer/**`. Business logic must not drift back into template-era top-level directories.

## Current Frontend Routes

- `/`
- `/auth/sign-in`
- `/auth/sign-up`
- `/dashboard/admin/users`
- `/dashboard/admin/roles`
- `/dashboard/admin/menus`
- `/dashboard/admin/depts`
- `/dashboard/admin/posts`
- `/dashboard/admin/dicts`
- `/dashboard/admin/configs`
- `/dashboard/admin/notices`
- `/dashboard/admin/online`
- `/dashboard/admin/jobs`
- `/dashboard/admin/job-logs`
- `/dashboard/monitor/logs/operation-logs`
- `/dashboard/monitor/logs/login-logs`
- `/dashboard/profile`
- `/error/403`
- `/error/404`
- `/error/500`

`/dashboard` redirects to `/dashboard/admin/users`.

## Backend Public Endpoints

- `GET /health`
- `GET /metrics`
- `GET /openapi.json`
- `GET /docs`
- `GET /api/app/configs`
- `GET /api/auth/me`
- `POST /api/auth/sign-in`, `/api/auth/sign-up`, and `/api/auth/refresh`
- `GET /api/captcha/config`; `POST /api/captcha/challenge` and `/api/captcha/redeem`

Protected APIs are grouped under:

- `/api/account/*`
- `/api/navbar`
- `/api/system/users*`, `/api/system/roles*`, and `/api/system/menus*`
- `/api/system/depts*`, `/api/system/posts*`, `/api/system/dict-*`, and `/api/system/configs*`
- `/api/system/notices*`, `/api/system/online*`, and `/api/system/dashboard`
- `/api/system/jobs*` and `/api/system/job-logs*`
- `/api/system/operation-logs*` and `/api/system/login-logs*`

## Backend Domain Scope

- authentication and authorization
- user, role, permission, menu, and organizational management
- dictionaries, runtime system configuration, notices, and online sessions
- scheduled jobs and execution logs
- operation and authentication audit logs
- captcha issuance and redemption

## Repository Layout

```text
.
├── apps
│   ├── backend         # Axum app entry and composition root
│   └── frontend        # Next.js admin app
├── crates
│   ├── audit           # operation and login audit bounded context
│   ├── audit_contract  # cross-context audit event and endpoint contracts
│   ├── captcha         # captcha capability
│   ├── client_info     # shared client information infrastructure
│   ├── config          # typed config loading and validation
│   ├── constants       # shared constants
│   ├── kernel          # shared low-level primitives
│   ├── rbac            # RBAC bounded context
│   ├── scheduler       # scheduled job bounded context
│   ├── storage         # database and storage foundation
│   ├── system          # system administration bounded context
│   ├── tracing         # tracing and metrics helpers
│   ├── types           # shared transport DTOs
│   └── user            # user bounded context
├── config              # YAML runtime configuration
├── migrations          # SQLx migration files
├── compose.yaml        # local Postgres and Redis
└── justfile            # Rust-focused dev commands
```

## Architecture Guardrails

These rules are part of the repository contract:

- Frontend must stay inside the current FSD layers under `apps/frontend/src`.
- Do not reintroduce top-level template directories such as `sections`, `layouts`, `actions`, `types`, `components`, `routes`, `locales`, `theme`, `assets`, `lib`, `auth`, or `global-config` as parallel business structure.
- Backend must stay inside DDD + Clean Architecture boundaries.
- `apps/backend` is not a domain bucket; it is the composition root.
- Generic crates such as `config`, `storage`, `types`, `constants`, `kernel`, and `tracing` are not substitutes for bounded contexts.

## Quick Start

### Prerequisites

- Rust toolchain
- Node.js 22+
- pnpm 10+
- Docker

### 1. Install dependencies

```bash
pnpm install
```

### 2. Start infrastructure

```bash
cp .env.example .env
# Set TACO_POSTGRES_PASSWORD in the ignored .env file, then start the services.
docker compose up -d
```

This starts:

- PostgreSQL on `localhost:5435`
- Redis on `localhost:6381`

### 3. Create deployment configuration

```bash
cp config/config.example.yaml config/config.local.yaml
```

Set the PostgreSQL and Redis connection credentials, generate a deployment-specific `jwt.secret` of at least 32 UTF-8 bytes, set `captcha.cloudflare_turnstile.secret_key` before selecting that provider, and replace the example CORS origin. The example intentionally contains no usable credentials or secrets. `config/config.local.yaml` is ignored by Git. Every backend invocation must select its deployment file with `--config <path>`; no configuration path is searched implicitly.

### 4. Apply database migrations

```bash
cargo run -p backend -- --config config/config.local.yaml migration up
```

### 5. Bootstrap the first system administrator

```bash
read -rs BOOTSTRAP_PASSWORD
printf '\n'
printf '%s\n' "$BOOTSTRAP_PASSWORD" | cargo run -p backend -- --config config/config.local.yaml bootstrap-admin --username root-admin --email root-admin@example.com
unset BOOTSTRAP_PASSWORD
```

The command accepts the password only from stdin and fails when any non-deleted user bound to the `admin` role already exists, including a disabled user. Existing ordinary users do not block the first bootstrap.

### 6. Start the backend

```bash
cargo run -p backend -- --config config/config.local.yaml
```

Backend default address:

- `http://127.0.0.1:3000`

### 7. Start the frontend

```bash
pnpm dev:frontend
```

Frontend default address:

- `http://127.0.0.1:8082`

By default, the frontend calls the backend at `http://127.0.0.1:3000`. Override it with `NEXT_PUBLIC_SERVER_URL` when needed.

## Useful Commands

### Rust workspace

Before running Rust tests, create the ignored `config/config.local.yaml` from the example and set its PostgreSQL fields to a reachable server whose configured user may create and drop databases. Migration integration tests load the complete typed YAML path from `TACO_TEST_CONFIG`; relative values resolve from the workspace root. The repository `.cargo/config.toml` supplies only the relative path `config/config.local.yaml`, never credentials or a fallback connection.

```bash
just check
just build
just test
just backend-migration config/config.local.yaml "status"
just backend-migration config/config.local.yaml "fresh"
```

### Frontend workspace

```bash
pnpm dev:frontend
pnpm build:frontend
pnpm lint:frontend
```

## Configuration

`config/config.example.yaml` documents the required schema without credentials. Each deployment supplies its own ignored configuration file and passes that path explicitly with `--config <path>`.

Important areas:

- server host and port
- PostgreSQL connection
- Redis connection
- JWT secret and token TTL
- Cloudflare Turnstile server-side secret
- auth whitelist and refresh Cookie scope
- PostgreSQL online-session cleanup interval and batch size
- CORS
- HTTP timeout and compression
- durable audit outbox delivery and client IP location resolution
- tracing and file logging

Required audit and client-information settings:

- `audit.outbox.worker_count`: number of durable audit outbox consumers
- `audit.outbox.claim_batch_size`: maximum outbox rows claimed by one consumer transaction
- `audit.outbox.poll_interval_ms`: delay before polling an empty outbox again
- `audit.outbox.lease_duration_ms`: time after which an abandoned claimed record can be claimed again
- `audit.outbox.retry_delay_ms`: delay before retrying a projection failure
- `audit.outbox.cleanup_interval_ms`: interval for removing completed delivery receipts
- `audit.outbox.cleanup_batch_size`: maximum completed receipts removed by one cleanup transaction
- `audit.outbox.processed_retention_days`: retention period for completed receipts, not final audit logs
- `client_info.ip_location.request_timeout_ms`: total, connect, and read timeout for one IP provider request

Required online-session lifecycle settings:

- `user.online_sessions.cleanup_interval_ms`: interval between independent expired-session cleanup cycles
- `user.online_sessions.cleanup_batch_size`: maximum expired sessions removed by one cleanup transaction

The typed YAML schema always requires `captcha.cloudflare_turnstile.secret_key` to be present, but it may remain blank while the CAP provider is selected. A non-blank value is required for Turnstile verification; a missing value fails that verification explicitly without fallback. The secret belongs exclusively in the deployment YAML. `sys.account.captchaConfig` contains only non-sensitive runtime settings, including the provider selection, Turnstile site key, theme, and size; it must never contain the private key.

Production traffic must reach the applications through a trusted reverse proxy. The proxy owns TLS termination and HSTS, enforces IP rate limits for sign-in, refresh, and captcha endpoints, strips client-supplied forwarding headers, and writes the canonical client IP headers consumed by the backend. The backend intentionally does not duplicate proxy-owned IP limiting or HSTS behavior.

Before deploying anywhere beyond local development, review and change:

- database credentials
- Redis connection settings
- JWT secret of at least 32 UTF-8 bytes
- Cloudflare Turnstile secret when that provider is selected
- CORS policy

## Notes

- `database.auto_migrate` defaults to `false`; use the explicit migration command unless a deployment deliberately enables startup migration.
- The in-development baseline and audit migrations were rewritten destructively; databases that applied their earlier checksums must be rebuilt with `just backend-migration config/config.local.yaml "fresh"`.
- RBAC cache is rebuilt during backend startup.
- The baseline data seeds roles, API permissions, menu sections, and menu items for the admin console, but no user accounts.
