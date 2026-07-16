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
- just

### 1. Install dependencies

```bash
pnpm install
```

### 2. Export the local process environment

The project reads runtime secrets only from the environment inherited by the process. Export the variables required by `config/config.local.yaml` in the current shell, IDE run configuration, `direnv` process environment, or a secret manager:

- `TACO_DATABASE_PASSWORD`: required, non-empty PostgreSQL password
- `TACO_REDIS_USERNAME`: required to exist; set it to an empty string when unused
- `TACO_REDIS_PASSWORD`: required to exist; set it to an empty string when unused
- `TACO_REDIS_DATABASE`: required to exist; set it to an empty string to use the Redis server default
- `TACO_JWT_SECRET`: required, non-empty JWT signing secret of at least 32 UTF-8 bytes
- `TACO_TURNSTILE_SECRET_KEY`: required to exist; set it to an empty string while Turnstile is not selected

Example for the local Compose services:

```bash
export TACO_DATABASE_PASSWORD='<local-postgres-password>'
export TACO_REDIS_USERNAME=''
export TACO_REDIS_PASSWORD=''
export TACO_REDIS_DATABASE=''
export TACO_JWT_SECRET='<at-least-32-UTF-8-bytes>'
export TACO_TURNSTILE_SECRET_KEY=''
```

Environment files are not a runtime input. The backend does not load them, the frontend refuses to start or build when a prohibited `.env*` file exists, and the supported Compose commands disable automatic env-file loading.

### 3. Start infrastructure

```bash
just services-up
```

This starts:

- PostgreSQL on `localhost:5435`
- Redis on `localhost:6381`

`just services-up` and `just services-down` force `COMPOSE_DISABLE_ENV_FILE=1` and clear inherited `COMPOSE_ENV_FILES`. They are the supported project entry points for Compose. PostgreSQL uses the same `TACO_DATABASE_PASSWORD` as the local backend profile.

### 4. Apply database migrations

```bash
just backend-migration config/config.local.yaml "up"
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
just run-local
```

Backend default address:

- `http://localhost:3000`

### 7. Start the frontend

```bash
pnpm dev:frontend
```

Frontend default address:

- `http://localhost:8082`

By default, the frontend calls the backend at `http://localhost:3000`. Local browser traffic must use `localhost` for both applications so the strict refresh Cookie remains same-site. Requests to the frontend through `127.0.0.1:8082` redirect to the canonical localhost origin. Supply `NEXT_PUBLIC_SERVER_URL` through the frontend process environment only when the configured frontend and backend origins remain same-site.

## Useful Commands

### Rust workspace

Before running Rust tests, export the local profile variables listed above. The PostgreSQL user must be able to create and drop test databases. Migration integration tests load the complete typed YAML path from `TACO_TEST_CONFIG`; relative values resolve from the workspace root. The repository `.cargo/config.toml` supplies only `config/config.local.yaml`, never credentials or fallback values.

```bash
just check
just build
just test
just run-local
just run-dev
just run-prod
just backend-migration config/config.local.yaml "status"
just backend-migration config/config.local.yaml "fresh"
just services-up
just services-down
```

### Frontend workspace

```bash
pnpm dev:frontend
pnpm build:frontend
pnpm lint:frontend
```

## Configuration

The repository tracks four explicit YAML files:

- `config/config.local.yaml`: developer workstation profile with fixed loopback services
- `config/config.dev.yaml`: shared remote development and integration profile
- `config/config.prod.yaml`: production profile
- `config/config.example.yaml`: fully commented production-security reference

Every backend invocation must pass `--config <path>` explicitly; no profile is inferred and no default path is searched. `${VAR}` must occupy the complete YAML scalar. Missing variables, embedded expressions such as `prefix-${VAR}`, default expressions such as `${VAR:-value}`, invalid UTF-8, and values that cannot be parsed as the target field type fail startup explicitly.

The selected profile requires every variable it references to exist. An explicitly empty value becomes `None` only for optional fields such as the Redis username, password, and database. Empty required strings remain empty and then fail field validation when the field does not permit them.

All profiles use these variables:

| Variable | Empty allowed | Meaning |
| --- | --- | --- |
| `TACO_DATABASE_PASSWORD` | No | PostgreSQL password |
| `TACO_REDIS_USERNAME` | Yes | Optional Redis username |
| `TACO_REDIS_PASSWORD` | Yes | Optional Redis password |
| `TACO_REDIS_DATABASE` | Yes | Optional Redis database number |
| `TACO_JWT_SECRET` | No | JWT signing secret, at least 32 UTF-8 bytes |
| `TACO_TURNSTILE_SECRET_KEY` | Yes | Turnstile private key; empty only while Turnstile is not selected |

The `dev`, `prod`, and production-baseline `example` profiles additionally use:

| Variable | Type and meaning |
| --- | --- |
| `TACO_DATABASE_HOST` | Non-empty PostgreSQL host |
| `TACO_DATABASE_PORT` | PostgreSQL port integer |
| `TACO_DATABASE_USERNAME` | Non-empty PostgreSQL username |
| `TACO_DATABASE_NAME` | Non-empty PostgreSQL database name |
| `TACO_REDIS_HOST` | Non-empty Redis host |
| `TACO_REDIS_PORT` | Redis port integer |
| `TACO_ADMIN_ORIGIN` | Exact HTTPS frontend Origin allowed by CORS |
| `TACO_LOG_DIRECTORY` | Non-empty directory for daily rolling log files |
| `TACO_AVATAR_DIRECTORY` | Non-empty persistent avatar storage path |

Connection URLs are constructed from typed PostgreSQL and Redis fields. `database.url`, `redis.url`, aliases, profile-prefixed variable names, and env-file compatibility paths are not supported. PostgreSQL schemes are restricted to `postgres` and `postgresql`; Redis schemes are restricted to `redis` and `rediss`; Redis protocol values are restricted to `resp2` and `resp3`.

The local profile explicitly disables PostgreSQL TLS and uses plain Redis only on loopback. The `dev`, `prod`, and `example` profiles set PostgreSQL `ssl_mode` to `verify-full` and use `rediss` with certificate and hostname verification.

SQLx-compatible ambient connection variables conflict with the typed YAML contract and must be unset: `PGHOSTADDR`, `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`, `PGSSLMODE`, `PGSSLROOTCERT`, `PGSSLCERT`, `PGSSLKEY`, `PGAPPNAME`, `PGOPTIONS`, and `PGPASSFILE`.

Important areas:

- server host and port
- PostgreSQL connection
- Redis connection
- JWT signing secret
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

The typed YAML schema always requires `captcha.cloudflare_turnstile.secret_key` to be present. Its value comes only from `TACO_TURNSTILE_SECRET_KEY` and may remain explicitly empty while the CAP provider is selected. Turnstile verification requires a non-empty value and fails explicitly without fallback. `sys.account.captchaConfig` contains only non-sensitive runtime settings, including provider selection, Turnstile site key, theme, and size; it never contains the private key.

Production traffic must reach the applications through a trusted reverse proxy. The proxy owns TLS termination and HSTS, enforces IP rate limits for sign-in, refresh, and captcha endpoints, strips client-supplied forwarding headers, and writes the canonical client IP headers consumed by the backend. It must not expose `/docs` or `/openapi.json`; those routes are also absent from the production authentication whitelist. The backend intentionally does not duplicate proxy-owned IP limiting or HSTS behavior.

The production frontend and API must use HTTPS and remain in the same schemeful site, for example `admin.example.com` and `api.example.com`. Refresh Cookies are host-only, `Secure`, `SameSite=Strict`, and scoped to `/api/auth`; cross-site deployment topologies are unsupported.

Before deploying anywhere beyond local development, review and change:

- database host and credentials
- Redis connection settings
- JWT secret of at least 32 UTF-8 bytes
- Cloudflare Turnstile secret when that provider is selected
- exact HTTPS admin Origin

## Notes

- All tracked profiles set `database.auto_migrate` to `false`; apply migrations explicitly before starting a new release.
- The in-development baseline and audit migrations were rewritten destructively; databases that applied their earlier checksums must be rebuilt with `just backend-migration config/config.local.yaml "fresh"`.
- RBAC cache is rebuilt during backend startup.
- The baseline data seeds roles, API permissions, menu sections, and menu items for the admin console, but no user accounts.
