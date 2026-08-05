# Production Docker Deployment

Taco's production image contains the statically exported frontend and one
`taco` executable. PostgreSQL and Redis are external services; the production
Compose file does not create them. Startup configuration comes from a host YAML
file. The container neither creates nor changes that configuration.

## Configuration File

Create the host configuration from `config/config.example.yaml` in the release
package or repository:

```bash
sudo install -d -m 0750 /etc/taco
sudo cp config/config.example.yaml /etc/taco/config.yaml
sudo chmod 0600 /etc/taco/config.yaml
```

To generate the JWT signing secret, build the image and run the YAML-independent
secret command:

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco secret generate-jwt
```

Copy the command's sole output in full into `jwt.secret` in
`/etc/taco/config.yaml`. Do not commit the secret or pass it through command
arguments. From a checked-out Rust toolchain, the equivalent command is:

```bash
cargo run -p backend --bin taco -- secret generate-jwt
```

Use a controlled editor to replace every `<...>` placeholder. A production
configuration must meet these constraints:

- `data_directory` may be absolute or relative; a relative path is resolved from
  the YAML file's directory. Retain the template's `../local-data`: Compose
  mounts the YAML at `/app/config/config.yaml`, where it resolves to
  `/app/local-data`. The named `taco-data` volume persists that directory, and
  the Local File Provider uses `/app/local-data/files`.
- Use a container-listenable `server.host` and a `server.port` that matches the
  Compose-published port.
- Supply real external `database`, `redis`, and `jwt.secret` values. The
  `jwt.secret` must contain at least 32 UTF-8 bytes. Do not commit real
  configuration or credentials.
- Explicitly set `database.auto_migrate` to `false` for production.

Configuration loading is strict: every field must be present. Unknown fields,
unreplaced `<...>` placeholders, or a blank required value make Taco exit with
an error. YAML has no environment-variable interpolation or implicit defaults.
Restart Taco after changing
`/etc/taco/config.yaml`.

## First Start

`compose.production.yaml` mounts host `/etc/taco/config.yaml` read-only at
`/app/config/config.yaml` and starts `taco --config /app/config/config.yaml`.
The example disables automatic migration, so a new database needs a schema and
an enabled system administrator before Taco can bind its HTTP port.

Build the Compose service image, then inspect and apply migrations with the
same read-only configuration:

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration status
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration up
```

Create the administrator explicitly before the first start:

```bash
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml administrator bootstrap --username <username> --email <email> --password-stdin
```

`--password-stdin` consumes the first password line from standard input. A
password is neither accepted as a command argument nor written to YAML or
command output. The command is allowed only when no enabled user is bound to
the built-in `admin` (`system=true`) role, and it creates the user and role
binding atomically.

Start the service after that initialization:

```bash
docker compose -f compose.production.yaml up -d
```

The service publishes only `127.0.0.1:3000`. The Compose liveness probe targets
`/health`; `/ready` returns `200` once the HTTP service has started.
Configuration, schema, and dependency initialization complete before the
listener is bound, so `/ready` is not a continuous dependency health probe.

## Reverse Proxy Contract

Terminate TLS at the host-side reverse proxy and proxy its upstream requests to
`http://127.0.0.1:3000`. The proxy must remove client-supplied forwarding
headers and set the canonical `X-Forwarded-For`, `X-Forwarded-Host`, and
`X-Forwarded-Proto` values. Taco accepts these standard headers without a
trusted-proxy CIDR configuration.

Keep the browser and `/api` traffic on the same public origin. Taco owns
frontend security headers and same-origin API behavior; the proxy owns its
domain-specific TLS certificate, HSTS, and network policy.

The proxy must keep `/metrics`, `/docs`, and `/openapi.json` internal. Do not
route those paths from the public virtual host. Restrict metrics scraping and
API documentation access to the operator or private monitoring network.

## Migrations And Upgrades

Production configuration should disable automatic migration. Every schema
change for a deployed or data-retaining instance requires a new forward
migration. When upgrading the current Compose service image, build it first,
run the same `migration status` and `migration up` commands used for first
start, then restart Taco:

```bash
docker compose -f compose.production.yaml build taco
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration status
docker compose -f compose.production.yaml run --rm taco --config /app/config/config.yaml migration up
docker compose -f compose.production.yaml up -d --force-recreate taco
```

If no enabled user is bound to the built-in `admin` role, restore one with the
`administrator bootstrap` command from First Start before restarting. Taco
never creates an administrator from startup YAML and refuses to start without
an enabled administrator. If `database.auto_migrate` is explicitly set to
`true`, Taco applies forward migrations before accepting requests; production
should still use the explicit procedure above.

## Configuration And Data Relocation

Move the host `/etc/taco/config.yaml`, external PostgreSQL, Redis, and the
`taco-data` volume together when moving a server. On the new host, keep mounting
the configuration read-only at `/app/config/config.yaml`; with
`data_directory: ../local-data`, its runtime directory remains
`/app/local-data`. When database, Redis, listen address, or data directory
changes, update YAML and restart. Startup YAML has no online reload. Whether a
`sys_config` parameter takes effect online is defined by its owning feature; it
is not a substitute for startup YAML.

## Build Contract

The repository release command exports the frontend first and then enables the
Rust embedding feature:

```bash
just build-release
```

For local development, run the Next.js frontend separately and do not enable
the embedded frontend feature.
