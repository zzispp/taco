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

`database.auto_migrate` is a required boolean:

- With `true`, Taco applies forward migrations and validates the schema before it accepts requests.
- With `false`, Taco only validates the schema. Pending, dirty, or checksum-mismatched migrations make startup fail. Production should use `false` and run migrations as an explicit operator step.

The schema operator subcommands are `migration status` and `migration up`:

```bash
taco --config <CONFIG_PATH> migration status
taco --config <CONFIG_PATH> migration up
```

An unpublished development baseline with a rebuildable database may be changed destructively; rebuild that database and reapply every migration afterward. Every schema change for a deployed or data-retaining instance requires a new forward migration. Restart Taco after applying it so the process rebuilds its runtime dependencies against the validated schema. The administrator seed data creates only the system `admin` role and explicit menu bindings, not a user.

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

The example configuration defaults `database.auto_migrate` to `false`. In the first terminal, apply migrations, create the first administrator, and start the backend:

```bash
just backend-migration up
cargo run -p backend --bin taco -- --config config/config.yaml administrator bootstrap --username <username> --email <email> --password-stdin
just run-backend
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

Production Compose runs Taco only. PostgreSQL and Redis are external, operator-managed dependencies. Put production YAML at `/etc/taco/config.yaml` and retain the template's `data_directory: ../local-data`. Compose mounts the configuration at `/app/config/config.yaml`, so that relative path resolves to `/app/local-data`, which the named `taco-data` volume persists. Production should set `database.auto_migrate` to `false`: a new database needs explicit migration and administrator bootstrap before its first start, while an existing instance is restarted after migration during an upgrade.

Before editing `jwt.secret` in production YAML, build the image and generate a secret:

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco secret generate-jwt
```

Copy the output into `jwt.secret` in `/etc/taco/config.yaml`; do not commit that file or pass the secret through command arguments.

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
