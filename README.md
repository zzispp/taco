# Taco

Taco is a Rust and Next.js administrative application. The backend uses DDD
and Clean Architecture; the frontend uses Feature-Sliced Design (FSD).

Taco is installed from its first browser visit. PostgreSQL, Redis, and the
initial super-administrator are configured in the setup wizard rather than in
a tracked runtime configuration file.

## Highlights

- Rust/Axum backend with PostgreSQL, Redis, SQLx migrations, and typed APIs
- Next.js 16 admin frontend, statically exported and embedded in the release
  executable
- User, RBAC, system, scheduler, audit, observability, and CAPTCHA contexts
- Encrypted installation state with one operator-owned root key
- A three-step first-visit setup flow with connection tests
- A marker-based installation super-administrator, independent of business
  roles and permissions

## Installation Model

### Bootstrap inputs

The executable has only these bootstrap inputs:

| Input | Source | Required | Meaning |
| --- | --- | --- | --- |
| Data directory | `--data-dir` or `TACO_DATA_DIR` | Serve, migration, reset | Holds the encrypted installation state and uploads. |
| Configuration-encryption key | `--config-encryption-key` or `TACO_CONFIG_ENCRYPTION_KEY` | Serve and migration | A 32-byte Base64URL root key that encrypts installation state. |
| Listener address | `--listen` or `TACO_LISTEN_ADDR` | No | Defaults to `0.0.0.0:3000`. |

Each input accepts exactly one source: command-line and environment sources
are mutually exclusive. `taco secrets generate` requires neither a data
directory nor an existing installation and prints a root-key assignment:

```bash
cargo run --quiet -p backend --bin taco -- secrets generate
```

Keep the emitted `TACO_CONFIG_ENCRYPTION_KEY` outside the repository. It is
the only operator-supplied secret required to start Taco. PostgreSQL and Redis
credentials are entered in the wizard and are persisted only in encrypted
installation state.

### First visit

Start Taco with a data directory and root key:

```bash
export TACO_DATA_DIR="$PWD/.local/taco-data"
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
cargo run -p backend --bin taco --
```

When installation state is absent, Taco starts in setup mode. `/health` is
live, `/ready` returns `503`, and the public `/setup` page provides three
steps:

1. Enter and test PostgreSQL host, port, database, username, password, and
   TLS choice.
2. Enter and test Redis host, port, optional username/password/database, and
   TLS choice.
3. Create the initial installation super-administrator. The optional advanced
   section uses backend-provided defaults for HTTP, metrics, session cleanup,
   audit outbox, IP-location timeout, scheduler, and Redis key prefix.

The final submission repeats connection checks, clears the selected PostgreSQL
`public` schema and runs `FLUSHALL` against the selected dedicated Redis
instance. `FLUSHALL` clears every logical database in that instance, regardless
of the selected database number or key prefix. Taco then applies initial
migrations, creates the marked super-administrator, encrypts the installation
profile, and exits for a supervised restart. Restart the local process after
completing the wizard. Production Compose restarts it automatically.

The marked super-administrator has no preassigned business role. It bypasses
business permission checks through the installation-owner marker. Create
business administrators from user management and explicitly assign their roles
and permissions there.

Infrastructure settings are immutable after setup. They cannot be edited from
the admin UI. Operational settings that belong to `sys_config` retain their
own management APIs.

### Resetting an installation

To deliberately reset an installation, stop Taco and delete its local encrypted
installation state:

```bash
export TACO_DATA_DIR="$PWD/.local/taco-data"
cargo run -p backend --bin taco -- installation reset --confirm-reset
```

The command does not require the former root key. Start Taco again with a root
key to enter setup mode. When the wizard submits verified PostgreSQL and Redis
connections, it clears the selected PostgreSQL `public` schema and executes
`FLUSHALL` on the dedicated Redis instance before it creates a fresh
installation owner. `FLUSHALL` clears every logical database in that Redis
instance. Uploaded files in the data directory remain outside this database and
Redis reset.

### Migrations after installation

Initial setup applies the schema automatically. Later upgrades support only
forward migration operations and derive the database connection from encrypted
installation state:

```bash
just backend-migration status
just backend-migration up
```

Both commands require the same data directory and root key environment as the
running server. There is no reverse or destructive migration command.

## Local Development

### Prerequisites

- Rust toolchain
- Node.js 22+
- pnpm 10+
- Docker and Docker Compose
- just

Install frontend dependencies:

```bash
pnpm install
```

Start local PostgreSQL and Redis. `TACO_DATABASE_PASSWORD` is used only by the
development Compose service, not as Taco runtime configuration:

```bash
export TACO_DATABASE_PASSWORD='<local PostgreSQL password>'
just services-up
```

Start Taco as shown in [First visit](#first-visit), then start the standalone
Next.js frontend:

```bash
pnpm dev:frontend
```

The frontend runs at `http://localhost:8082` and proxies same-origin `/api/*`
requests to `http://localhost:3000`. Set the server-only
`TACO_DEV_BACKEND_URL` when the development backend is elsewhere. The browser
does not receive a separate API origin.

For the local Compose services, use these wizard values:

| Service | Host | Port | TLS |
| --- | --- | --- | --- |
| PostgreSQL | `localhost` | `5435` | Disabled |
| Redis | `localhost` | `6381` | Disabled |

Use PostgreSQL username `postgres`, database `postgres`, and the value of
`TACO_DATABASE_PASSWORD`. Redis has no password in the supplied local Compose
stack.

## Production Delivery

`just build-release` exports the frontend and compiles one Taco executable
with the embedded assets:

```bash
just build-release
```

The production Compose file runs only Taco. PostgreSQL and Redis remain
external services, while the `taco-data` volume stores encrypted installation
state and uploads. Follow [Production Docker Deployment](docs/deployment.md)
for root-key generation, first start, upgrades, reset, and the complete
Compose contract.

Place a TLS-terminating reverse proxy in front of the production container and
keep the browser and API on one public origin. The proxy must strip
client-supplied forwarding headers and write canonical `X-Forwarded-For`,
`X-Forwarded-Host`, and `X-Forwarded-Proto` headers. Taco accepts those
standard headers directly; no proxy-network range is configured. Refresh
cookies are host-only, `HttpOnly`, `SameSite=Strict`, scoped to `/api/auth`,
and gain `Secure` only for forwarded HTTPS.

Keep `/metrics`, `/docs`, and `/openapi.json` internal to the reverse proxy.
Do not route them from the public virtual host; restrict metrics scraping and
API documentation access to the operator or private monitoring network.

## Architecture

### Backend

`apps/backend` is the composition root. Business behavior stays in bounded
contexts:

- `crates/user`, `crates/rbac`, `crates/system`, `crates/scheduler`, and
  `crates/captcha`
- `crates/audit` and `crates/observability`
- `crates/config` for bootstrap inputs, encrypted installation state, and
  typed profile validation
- `crates/storage`, `crates/types`, `crates/constants`, `crates/kernel`, and
  `crates/tracing` for shared infrastructure

SQLx migrations are in `migrations/`. Static production assets are generated
from `apps/frontend` and embedded in the backend release binary.

### Frontend

The frontend lives in `apps/frontend/src` and follows this dependency
direction:

```text
app -> pages-layer/widgets/features/entities/shared
pages-layer -> widgets/features/entities/shared
widgets -> features/entities/shared
features -> entities/shared
entities -> shared
```

`/setup` is isolated from the authenticated application providers. In normal
mode its setup gate redirects visitors to sign-in; in setup mode it avoids
initializing application authentication and runtime-config providers.

## Operations and Validation

```bash
# Rust
just check
just build
just test
just quality-precommit
just quality-complete

# Frontend
pnpm lint:frontend
pnpm build:frontend
pnpm --filter frontend build:embedded

# Local services
just services-up
just services-down
```

`/health` is a liveness endpoint and returns `200` in both modes. `/ready`
returns `503` during setup and `200` only after the installed runtime is ready.
