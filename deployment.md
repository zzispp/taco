# Production Docker Deployment

Taco's production image contains the statically exported frontend and one
`taco` executable. PostgreSQL and Redis are external services; they are not
created by the production Compose file and their connection details are
entered in the first-visit setup wizard.

## Start

Generate a configuration root key before the first start. The command does
not require a data directory or an existing installation. With Docker only,
build the image once and run its operator command:

```bash
docker build --tag taco:local .
docker run --rm taco:local secrets generate
```

An operator using a checked-out Rust toolchain can run the equivalent
`cargo run --quiet -p backend --bin taco -- secrets generate` command.

Copy the generated value into the shell that starts Compose:

```bash
export TACO_CONFIG_ENCRYPTION_KEY='<generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d --build
```

The service publishes only `127.0.0.1:3000`. Visit the public site through
the host's HTTPS reverse proxy and complete the three setup steps. Taco stores
the encrypted installation state and mutable uploads in the named
`taco-data` volume mounted at `/data`.

The Compose liveness probe targets `/health`. During setup that endpoint is
healthy even though `/ready` remains unavailable until the installed runtime
can reach PostgreSQL and Redis.

## Reverse Proxy Contract

Terminate TLS at the host-side reverse proxy and proxy its upstream requests
to `http://127.0.0.1:3000`. The proxy must remove client-supplied forwarding
headers and set the canonical `X-Forwarded-For`, `X-Forwarded-Host`, and
`X-Forwarded-Proto` values. Taco accepts these standard headers without a
trusted-proxy CIDR configuration.

Keep the browser and `/api` traffic on the same public origin. Taco owns
frontend security headers and same-origin API behavior; the proxy owns its
domain-specific TLS certificate, HSTS, and network policy.

The proxy must keep `/metrics`, `/docs`, and `/openapi.json` internal. Do not
route those paths from the public virtual host. Restrict metrics scraping and
API documentation access to the operator or private monitoring network.

## Operations

Apply forward migrations from the new image before starting an installed release. `run` creates a one-off operator container, so it does not depend on the normal service becoming ready before the migration is applied:

```bash
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml run --rm taco migration up
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d
```

To deliberately reset an instance, stop the service, generate and export a
new root key, then remove the encrypted installation state:

```bash
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml stop taco
docker run --rm taco:local secrets generate
export TACO_CONFIG_ENCRYPTION_KEY='<new generated Base64URL value>'
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml run --rm taco installation reset --confirm-reset
COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose -f compose.production.yaml up -d
```

The reset command does not need the former root key. After the next setup
submission revalidates its PostgreSQL and Redis inputs, Taco only performs a
destructive reset on a fresh target. If the selected PostgreSQL database already
contains Taco schema, setup rejects the request before PostgreSQL or Redis is
changed. `FLUSHALL` clears every logical database in the selected Redis instance,
regardless of the selected database number or key prefix, so a fresh installation
target must still be dedicated to Taco.

## Server Migration And State Recovery

Move the named `taco-data` volume, `TACO_CONFIG_ENCRYPTION_KEY`, PostgreSQL,
Redis, and uploads together when moving a server. An intact encrypted state file
starts the migrated installation directly; do not repeat web setup.

When only database or Redis endpoints change, mount the data volume and run
`installation reconfigure --connections <path>` with a JSON document containing
the `database` and `redis` objects from the installation profile. The command
checks the migrated schema, installation owner, and Redis before atomically
replacing the encrypted state.

When the state file is missing but the Taco database is intact, generate a
complete `InstallationProfile` JSON template with `installation profile template`,
fill in the former immutable configuration and current connections, then run
`installation recover --profile <path>`. Recovery verifies the same database
and Redis invariants, writes a new encrypted state file, and generates a fresh
JWT signing key. Existing browser sessions are intentionally invalidated; users
must sign in again.

If the former root key is lost but `installation-state.enc` remains, run
`installation reset --confirm-reset` first to remove only that encrypted state
file, then recover with a new root key. This action does not change PostgreSQL,
Redis, or uploads.

## Build Contract

The repository release command exports the frontend first and then enables the
Rust embedding feature:

```bash
just build-release
```

For local development, run the Next.js frontend separately and do not enable
the embedded frontend feature.
