# Taco

[简体中文](README.md)

Taco is an administrative application built with Rust/Axum and Next.js. Its backend follows DDD and Clean Architecture, while its frontend follows Feature-Sliced Design (FSD). Production builds embed the statically exported frontend in the `taco` executable; development runs the frontend as a separate Next.js process.

## Overview

- PostgreSQL, Redis, SQLx migrations, and typed APIs
- User management, RBAC, system administration, scheduling, audit, observability, CAPTCHA, and file management
- Strict YAML startup configuration, explicitly selected with `taco --config <path>`
- Administrator access is determined only by RBAC roles and menu bindings; no identity marker bypasses authorization
- Simplified Chinese, English, and Traditional Chinese UI and API error responses

## Layout And Architecture

`apps/backend` is the composition root only: it owns startup, dependency wiring, routes, and migration commands. It must not contain domain business rules.

Backend bounded contexts:

- `crates/audit`, `crates/observability`, `crates/user`, `crates/rbac`, `crates/system`, `crates/scheduler`, `crates/captcha`, and `crates/file`.
- Contexts separate `domain`, `application`, `infra`, and `api`; `apps/backend` only composes capabilities provided by those contexts.
- `crates/audit_contract` owns cross-context audit contracts; `crates/client_info`, `crates/config`, `crates/storage`, `crates/types`, `crates/constants`, `crates/kernel`, and `crates/tracing` provide shared foundations; `crates/rbac_macros` and `crates/scheduler_macros` are supporting macro crates.

SQLx migrations live in `migrations/`. Release builds generate `apps/frontend/out` and embed it through the `embedded-frontend` feature.

The frontend lives in `apps/frontend/src` and has this fixed dependency direction:

```text
app -> pages-layer/widgets/features/entities/shared
pages-layer -> widgets/features/entities/shared
widgets -> features/entities/shared
features -> entities/shared
entities -> shared
```

`src/app/**/page.tsx` contains only route entry, metadata, and guards; page composition belongs in `pages-layer`.

## Contribution Rules

- Domain rules belong only to their owning bounded context. Shared crates, DTOs, HTTP handlers, and the composition root must not absorb business rules.
- Startup infrastructure configuration comes only from the YAML selected with `--config`; mutable business and runtime parameters belong only in `sys_config`. One semantic has one active source.
- An unpublished migration baseline with a rebuildable development database may be changed destructively only by an explicit project decision. Schema changes for deployed or data-retaining instances require a forward migration. Migrations and seed data must provide valid defaults.
- Put UI copy in its existing i18n namespace rather than hardcoding it in components. URL locale and backend wire-locale mapping derive only from `locale-contract.json`.
- Run the Rust quality gate before committing. [AGENTS.md](AGENTS.md) is the complete source for architecture, configuration, internationalization, and test rules.

## Startup Configuration

`config/config.example.yaml` defines the complete configuration shape. The actual runtime file is the untracked `config/config.yaml`:

```bash
mkdir -p config
cp config/config.example.yaml config/config.yaml
```

Replace every `<...>` placeholder in the example with a real value. Configuration loading is strict:

- Every field must be supplied explicitly. Unknown or missing fields, a repeated `--config`, blank values, and unreplaced `<...>` placeholders make startup fail.
- YAML has no environment-variable interpolation and no implicit defaults. Optional Redis fields must still be written explicitly as a value or `null`.
- `data_directory` may be absolute or relative. A relative path is resolved from the YAML file's directory, and runtime receives only the resulting absolute path. The repository template's `../local-data` resolves to `./local-data` at the repository root. The Local File Provider always uses `<data_directory>/files` and maintains `objects/`, `parts/`, and `derivatives/` below it; there is no second configurable local-storage root.
- YAML contains `server`, `data_directory`, `database`, `jwt`, `redis`, `user.online_sessions`, `http`, `metrics`, `audit`, `client_info`, and `scheduler`. Restart Taco after changing YAML; these values are not reloaded at runtime.
- `database.pool` must explicitly set `max_connections`, `acquire_timeout_ms`, `idle_timeout_ms`, and `max_lifetime_ms`. They respectively limit pool capacity, the maximum wait for a connection, idle connection reaping, and one connection's lifetime; all values are milliseconds and must be greater than zero.
- `database.session` must explicitly set `application_name`, `statement_timeout_ms`, `lock_timeout_ms`, and `idle_in_transaction_session_timeout_ms`. Taco applies them when every connection is established to identify the connection and bound statement execution, lock waits, and idle transaction occupancy; timeout values are milliseconds and must be greater than zero.
- Pool and session parameters are startup infrastructure configuration. Changing them requires restarting the service or rerunning the corresponding migration command; they are never read from a connection URL, environment variable, or implicit default.

Generate `jwt.secret` with:

```bash
cargo run -p backend --bin taco -- secret generate-jwt
```

The command neither reads nor changes YAML. Copy its sole output into `jwt.secret` in `config/config.yaml`; do not commit the secret or pass it as a command argument.

Always supply the configuration path when starting the service:

```bash
taco --config <CONFIG_PATH>
```

The repository-local development equivalent is:

```bash
cargo run -p backend --bin taco -- --config config/config.yaml
```

## Migrations And Initial Data

Service startup never applies migrations automatically. It uses only the runtime YAML to establish a least-privilege connection and check schema readiness; pending, dirty, or checksum-mismatched migrations make startup fail explicitly.

Production must maintain two independent, complete strict YAML files:

- The runtime YAML contains only the least-privilege database identity used by Taco. It must not have `CREATE`, `ALTER`, `DROP`, database-creation, or migration-ledger permissions, and it is the only file mounted into the long-running service.
- The migration YAML is used only by one-shot `migration status`/`migration up` commands and contains an isolated migration identity with elevated schema permissions. Do not mount it into the long-running service, commit it, or place its credentials in the runtime YAML. Pool and session values may be sized independently for the migration window.

Both YAML files must contain every field from the template because configuration parsing rejects missing and unknown fields. Production PostgreSQL connections must use `database.ssl_mode: verify-full` with the issuing CA trusted by the runtime; `verify-ca` or disabled TLS is not an acceptable production identity-validation setting. The certificate hostname must match `database.host`.

The schema operator subcommands are `migration status` and `migration up`:

```bash
taco --config <MIGRATION_CONFIG_PATH> migration status
taco --config <MIGRATION_CONFIG_PATH> migration up
```

Commands return success or failure directly to the caller; after a failed non-transactional online migration, never edit `_sqlx_migrations` by hand. An unpublished development baseline with a rebuildable database may be changed destructively; rebuild that database and reapply every migration afterward. Every schema change for a deployed or data-retaining instance requires a new forward migration. Restart Taco with the runtime YAML after applying it so the process rebuilds its runtime dependencies against the validated schema. The administrator seed data creates only the system `admin` role and explicit menu bindings, not a user.

For a first deployment, or recovery when no enabled user is bound to the built-in `admin` (`system=true`) role, explicitly create the administrator before starting the service:

```bash
taco --config <path> administrator bootstrap --username <username> --email <email> --password-stdin
```

`--password-stdin` consumes the first password line from standard input. The command accepts no password argument and never writes a password to YAML or command output.

The command succeeds only when the database has no enabled user bound to the built-in `admin` role, then creates the user and role binding in one database transaction. Service startup never creates or recovers an administrator automatically; it fails explicitly when that administrator is absent. Administrator users, role bindings, and data scopes are always managed by database RBAC.

## Local Development

### Prerequisites

- Rust toolchain (the workspace uses edition 2024)
- Node.js `>=22.12.0`
- pnpm `10.33.4`
- Docker and Docker Compose
- [just](https://github.com/casey/just)

Install frontend dependencies:

```bash
pnpm install
```

Create the local YAML file and replace every placeholder. The template's `data_directory: ../local-data` already resolves to `./local-data` at the repository root, so no manual data-directory value is needed. The default development Compose services use PostgreSQL at `127.0.0.1:5435` and Redis at `127.0.0.1:6381`; set `database.password` to the same local value used by Compose and generate an independent, real `jwt.secret` of at least 32 UTF-8 bytes with the command above. `TACO_DATABASE_PASSWORD` is used only to create the development PostgreSQL container, not as Taco runtime configuration:

```bash
mkdir -p config
cp config/config.example.yaml config/config.yaml
export TACO_DATABASE_PASSWORD='<LOCAL_POSTGRESQL_PASSWORD>'
just services-up
```

Rust integration tests read their PostgreSQL administrative connection from local `config/config.yaml`. Each test creates, connects to, and drops an isolated temporary database; it does not run migrations or write business tables in the database named by `database.name`. The configured PostgreSQL user must be allowed to create databases, terminate connections, and drop databases.

For local development, copy the template as the runtime configuration; a local database user may temporarily have both runtime and migration permissions, but production files must remain separate. In the first terminal, apply migrations, create the first administrator, and start the backend:

```bash
cargo run -p backend --bin taco -- --config config/config.yaml migration up
cargo run -p backend --bin taco -- --config config/config.yaml administrator bootstrap --username <username> --email <email> --password-stdin
cargo run -p backend --bin taco -- --config config/config.yaml
```

In the second terminal, start the standalone frontend:

```bash
pnpm dev:frontend
```

The frontend runs at `http://localhost:8082` and proxies same-origin `/api/*` to `http://localhost:3000`. The development backend does not embed static frontend assets. Set the server-only `TACO_DEV_BACKEND_URL` on the Next.js process when the backend uses another origin.

Stop local dependencies:

```bash
just services-down
```

## Production Delivery

Build the release executable:

```bash
just build-release
```

Production Compose runs Taco only. PostgreSQL and Redis are external, operator-managed dependencies. Put the least-privilege runtime YAML at `/etc/taco/runtime.yaml` and the separate elevated migration YAML at a more restricted path such as `/etc/taco/migration.yaml`; keep both files mode `0600` and readable only by the command that needs them. Compose mounts only the runtime YAML into the long-running service. A new database must be migrated explicitly with the migration YAML before administrator bootstrap and service start; an existing instance is migrated first and restarted with the runtime YAML during an upgrade.

Before editing `jwt.secret` in production YAML, build the image and generate a secret:

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco secret generate-jwt
```

Copy the output into `jwt.secret` in the host runtime and migration YAML files; do not commit those files or pass the secret through command arguments.

Compose publishes only `127.0.0.1:3000`. The browser and `/api` must use one public origin. The proxy must strip client-supplied forwarding headers and write canonical `X-Forwarded-For`, `X-Forwarded-Host`, and `X-Forwarded-Proto` values. Do not expose `/metrics`, `/docs`, or `/openapi.json` publicly.

See [Production Docker Deployment](deployment.md) for Docker, reverse-proxy, upgrade, and configuration-change procedures.

## Common Commands And Validation

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

`just quality-precommit` runs formatting, Clippy, the workspace check, and tests. `just quality-complete` adds `cargo audit` and `cargo deny check`. Run the former before committing and the latter before completing Rust work.

`/health` is the liveness probe. `/ready` returns `200` once the HTTP service has started; configuration, schema, and dependency initialization complete before the listener is bound, so it is not a continuous dependency health probe.
