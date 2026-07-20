# Taco

[简体中文](README.md)

Taco is an administrative application built with Rust/Axum and Next.js. Its backend follows DDD and Clean Architecture, while its frontend follows Feature-Sliced Design (FSD). Production builds embed the statically exported frontend in the `taco` executable; development runs the frontend as a separate Next.js process.

## Overview

- PostgreSQL, Redis, SQLx migrations, and typed APIs
- User management, RBAC, system administration, scheduling, audit, observability, CAPTCHA, and installation
- Three-step browser installation with connection settings stored in encrypted installation state
- One marker-authorized installation owner, independent of business roles and permissions
- Simplified Chinese, English, and Traditional Chinese UI and API error responses

## Layout And Architecture

`apps/backend` is the composition root only: it owns startup, dependency wiring, routes, and migration commands. It must not contain domain business rules.

Backend bounded contexts:

- `crates/audit`, `crates/observability`, `crates/user`, `crates/rbac`, `crates/system`, `crates/scheduler`, `crates/captcha`, and `crates/installation`.
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

`src/app/**/page.tsx` contains only route entry, metadata, and guards; page composition belongs in `pages-layer`. The installation page is isolated from authentication and runtime-config providers.

## Contribution Rules

- Domain rules belong only to their owning bounded context. Shared crates, DTOs, HTTP handlers, and the composition root must not absorb business rules.
- Bootstrap infrastructure parameters belong in encrypted installation state; mutable business and runtime parameters belong only in `sys_config`. One semantic must have one active source.
- Add a migration for every schema change and never edit an applied migration. Migrations and seed data must provide valid defaults.
- Put UI copy in its existing i18n namespace rather than hardcoding it in components. Frontend languages are `cn`, `en`, and `tw`; backend wire locales are `zh-CN`, `en`, and `zh-TW`.
- Run the Rust quality gate before committing. [AGENTS.md](AGENTS.md) is the complete source for architecture, configuration, internationalization, and test rules.

## Bootstrap And Installation

### Bootstrap Inputs

| Input                             | Source                                                    | Used by                  | Meaning                                                   |
| --------------------------------- | --------------------------------------------------------- | ------------------------ | --------------------------------------------------------- |
| Data directory                    | `--data-dir` or `TACO_DATA_DIR`                           | Serve, migrations, reset | Stores `installation-state.enc` and uploads.              |
| Configuration encryption root key | `--config-encryption-key` or `TACO_CONFIG_ENCRYPTION_KEY` | Serve, migrations        | A 32-byte Base64URL key that encrypts installation state. |
| Listen address                    | `--listen` or `TACO_LISTEN_ADDR`                          | Serve                    | Optional; defaults to `0.0.0.0:3000`.                     |

The command line and environment cannot provide the same input together. `taco secrets generate` does not read bootstrap inputs; `taco installation reset` accepts only a data directory.

Generate a root key:

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
```

The command prints `TACO_CONFIG_ENCRYPTION_KEY=<value>`. Move this key with the data directory. A lost key cannot decrypt existing installation state; use the explicit recovery flow to rebuild state, which invalidates existing browser sessions.

### First Installation

When installation state is absent, the backend enters setup mode: `/health` returns `200` and `/ready` returns `503`. The release executable redirects `/` to `/cn/`; with the standalone development frontend, visit `http://localhost:8082/cn/setup/`.

The installation wizard performs these steps:

1. Enter and test PostgreSQL host, port, database, username, password, and TLS choice.
2. Enter and test Redis host, port, optional username/password/database, and TLS choice.
3. Create the initial installation owner. The advanced section uses backend-provided defaults for HTTP, metrics, session cleanup, the audit outbox, IP location, scheduling, and the Redis prefix.

Final submission validates the connections again, drops the selected PostgreSQL database's `public` schema, and executes `FLUSHALL` on the selected Redis instance. It then runs all initial migrations, creates the installation owner, atomically writes encrypted state, and requests process exit for restart. The PostgreSQL database and Redis instance must be dedicated to this Taco installation. `FLUSHALL` clears every logical database in that Redis instance, regardless of the selected database number or key prefix.

The installation owner has no preassigned business role, but its marker bypasses business permission checks. Create other administrators and assign their roles and permissions explicitly through user and RBAC management. PostgreSQL, Redis, JWT, and advanced installation values cannot be changed online after installation; existing `sys_config` parameters remain owned by their management APIs.

### Resetting An Installation

Stop Taco, then remove only the local encrypted installation state:

```bash
export TACO_DATA_DIR="$PWD/.local/taco-data"
cargo run -p backend --bin taco -- installation reset --confirm-reset
```

This command does not require the former root key and does not delete uploads in the data directory. Start again with a new root key to re-enter setup mode. If the selected PostgreSQL database already contains Taco schema, setup rejects the submission before PostgreSQL or Redis is changed.

### Server Migration And Recovery

Move `TACO_DATA_DIR`, the configuration root key, PostgreSQL, Redis, and uploads together when moving a server. With intact encrypted installation state, start the migrated instance directly; do not run web setup again.

When only PostgreSQL or Redis endpoints change, create a JSON file containing the `database` and `redis` objects and run:

```bash
cargo run -p backend --bin taco -- \
  --data-dir /var/lib/taco \
  --config-encryption-key "$TACO_CONFIG_ENCRYPTION_KEY" \
  installation reconfigure --connections /secure/taco-connections.json
```

The command preserves immutable configuration and the JWT signing key, while verifying the migrated schema, installation owner, and Redis before it atomically replaces encrypted state.

When the state file is missing but the Taco database is intact, generate a complete `InstallationProfile` template, fill in the former immutable configuration and current connections, then use it instead of setup. Recovery replaces the template JWT value:

```bash
cargo run -p backend --bin taco -- installation profile template > /secure/taco-installation-profile.json
```

When the former root key is lost but `installation-state.enc` remains, first run `installation reset --confirm-reset` to remove only that state file. It does not change PostgreSQL, Redis, or uploads. Then run recovery with a new root key.

Run recovery with that file:

```bash
cargo run -p backend --bin taco -- \
  --data-dir /var/lib/taco \
  --config-encryption-key "$TACO_CONFIG_ENCRYPTION_KEY" \
  installation recover --profile /secure/taco-installation-profile.json
```

Recovery verifies the same database and Redis invariants, writes new encrypted state, and rotates the JWT signing key. Users must sign in again.

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

Start local PostgreSQL and Redis. `TACO_DATABASE_PASSWORD` is only for the development Compose service, not Taco runtime configuration:

```bash
export TACO_DATABASE_PASSWORD='<local PostgreSQL password>'
just services-up
```

In the first terminal, generate a root key, export bootstrap inputs, and start the backend:

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
export TACO_DATA_DIR="$PWD/.local/taco-data"
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
cargo run -p backend --bin taco --
```

In the second terminal, start the standalone frontend:

```bash
pnpm dev:frontend
```

The frontend runs at `http://localhost:8082` and proxies same-origin `/api/*` to `http://localhost:3000`. Visit `http://localhost:8082/cn/setup/` to perform the first local installation; the frontend redirects between setup and the normal application according to installation state. The development backend does not embed static frontend assets, so `http://localhost:3000/setup` is not a development entry point. Set the server-only `TACO_DEV_BACKEND_URL` on the Next.js process when the backend uses another origin.

Wizard values for the local Compose services:

| Service    | Host        | Port   | TLS      |
| ---------- | ----------- | ------ | -------- |
| PostgreSQL | `localhost` | `5435` | Disabled |
| Redis      | `localhost` | `6381` | Disabled |

The PostgreSQL username and database are both `postgres`; its password is the value of `TACO_DATABASE_PASSWORD`. The provided Redis service has no password. The frontend accepts inherited process environment only and rejects `.env` and `.env.*` files in the workspace root or `apps/frontend`.

Stop local services:

```bash
just services-down
```

## Migrations

First installation applies all migrations automatically. After an instance has been installed, use the same data directory and root key to inspect or apply forward migrations:

```bash
just backend-migration status
just backend-migration up
```

The operator CLI exposes only `status` and `up`; it has no rollback or database-reset command. Normal runtime explicitly refuses to start with pending migrations, dirty migrations, or checksum mismatches. For development changes, add migrations and required seed/test coverage instead of editing already-applied migration files.

## Production Delivery

Build the release executable:

```bash
just build-release
```

Production Compose runs Taco only. PostgreSQL and Redis are external, operator-managed dependencies. Export a root key before the first start:

```bash
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d --build
```

Compose mounts the `taco-data` volume at `/data` and publishes only `127.0.0.1:3000`. Complete installation through the site behind an HTTPS reverse proxy. The browser and `/api` must use one public origin. The proxy must strip client-supplied forwarding headers and write canonical `X-Forwarded-For`, `X-Forwarded-Host`, and `X-Forwarded-Proto` values. Do not expose `/metrics`, `/docs`, or `/openapi.json` publicly.

See [Production Docker Deployment](deployment.md) for the complete Docker, reverse-proxy, upgrade, and reset contract.

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

`/health` is the liveness probe and returns `200` in both setup and normal modes. `/ready` returns `200` only when installed runtime is ready; it returns `503` in setup mode.
