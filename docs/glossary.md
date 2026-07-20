# Glossary

## Compose Liveness Healthcheck

The production Compose probe of `GET /health`, chosen so an active setup
wizard remains healthy while `/ready` continues to represent normal-runtime
traffic readiness.

## Production Docker Installation Delivery

The Taco Docker packaging path with a multi-stage embedded-frontend build,
persistent installation-data volume, restart policy, and no backend DB, Redis,
or JWT environment configuration. Its production Compose example runs Taco
only; PostgreSQL and Redis are external administrator-managed dependencies.
The image retains its default runtime identity rather than imposing a fixed
non-root UID/GID or data-volume ownership contract. It uses
`restart: unless-stopped` to restart successfully completed setup into normal
runtime while honoring an explicit operator stop. The first-release Compose
example builds from the repository Dockerfile instead of referencing a
presumed registry image and publishes Taco only as `127.0.0.1:3000:3000` for
the host-side HTTPS reverse proxy. It unconditionally accepts standard
forwarded headers and has no Docker bridge CIDR setting. Its default durable
data store is the named volume `taco-data` mounted at `/data`, selected through
`TACO_DATA_DIR=/data`. Its root-key environment value is required even for
reset, where the operator supplies a newly generated value rather than the lost
former key. Taco documents the required proxy contract but ships no Nginx or
Caddy example because proxy domains are deployment-specific.

## No In-Place Root-Key Rotation

The first-release scope rule that configuration encryption keys are changed
only through configuration reset and a new setup cycle, not by a dedicated
state-reencryption command.

## Setup Liveness and Readiness

The health-check split in which `/health` reports a live setup process with
HTTP 200 and `status="setup"`, while `/ready` remains 503 until normal runtime
dependencies are ready.

## Multilingual Setup Wizard

The installation UI that supports Taco's Simplified Chinese, English, and
Traditional Chinese resources and language selection from its first screen.

## Embedded Frontend Cache Policy

The static-delivery cache split that makes hashed `/_next/static/*` artifacts
one-year immutable while HTML, 404 documents, and unversioned public assets use
`no-cache`.

## Embedded Frontend Security Headers

The Rust-owned CSP and related browser response-header policy used when Taco
serves its embedded static frontend. It uses same-origin API access, omits
Turnstile, and retains CAP-required WASM support.

## Embedded Frontend Real 404

The embedded static-site behavior that returns exported `404.html` with HTTP
404 for unknown frontend paths and never uses an `index.html` SPA fallback;
unknown API paths remain API 404s.

## Forward-Only Operator Migrations

The published migration CLI policy that permits only status inspection and
forward application of pending migrations, not rollback or database-reset
commands.

## Explicit Post-Installation Migration

The `taco migration up` deployment operation that decrypts installed
configuration to apply later schema migrations. Normal Taco runtime refuses a
database with pending migrations.

## Taco Operator Binary

The published `taco` executable built from the Cargo package named `backend`.
It owns normal serving, secret generation, installation reset, and deployment
operations.

## Setup-Mode Route Surface

The restricted route catalog available only before installation. Normal Taco
runtime retains only public `GET /api/setup/status`; all setup mutations and
connection tests are absent.

## Immutable Advanced Installation Settings

The bounded operational values chosen in the final setup panel that persist in
the installation profile and have no normal online editor in the first release.
They are distinct from existing online `sys_config` business parameters.

## Installation Profile

The single typed set of required operational startup values selected during
setup. It supplies visible defaults and accepts validated advanced overrides;
it has no user-facing profile-version concept.

## No Preseeded RBAC Roles

The fresh-installation rule that leaves no legacy `admin` or `common` role in
the final schema. The installation owner is marker-authorized, and all business
roles are created explicitly after installation.

## Fixed Session Cookie Policy

The non-configurable refresh-cookie policy: host-only, `HttpOnly`,
`SameSite=Strict`, and `/api/auth` scoped, with `Secure` enabled when Taco runs
behind a forwarded HTTPS request and omitted only when the forwarded scheme is
absent or HTTP, including direct local HTTP development.

## Unconditional Forwarded Header Trust

The rule that Taco always reads supported client-IP and external-scheme
forwarding headers without a proxy CIDR, environment-variable, or CLI setting.

## Proxy-Terminated Production TLS

The production topology in which a trusted reverse proxy terminates public
HTTPS and forwards HTTP to one Taco executable that serves both embedded pages
and API; local development bypasses the proxy over direct HTTP.

## CAP-Only Captcha

The initial-installation captcha scope in which Taco retains its built-in CAP
provider and removes Cloudflare Turnstile plus its external private-key
configuration.

## Scoped Advanced Setup Overrides

The optional final-step setup controls for operational sizing and behavior,
such as timeouts, workers, batches, metrics, and scheduler intervals. They
exclude bootstrap parameters, infrastructure connection inputs, and secrets.

## Bootstrap Startup Model

The sole Taco startup model consisting of explicit bootstrap arguments or
environment inputs, the encrypted installation state file, and the web setup
flow. It replaces legacy YAML profiles and infrastructure-variable
interpolation.

## Fresh Installation Only

The scope decision that Taco's web-installation architecture targets new
deployments only and provides no runtime conversion for legacy YAML,
environment-variable infrastructure settings, or role-based super-administrator
data.

## Installation Owner Table

The singleton `sys_installation_owner` database table whose sole row references
the one installation super administrator. It is independent of ordinary user
fields and assignable RBAC roles.

## Setup Restart Wait

The explicit post-installation frontend state that waits for the supervised
backend restart, probes setup status, routes to sign-in on normal runtime, and
shows a retryable connection failure when restart does not succeed.

## Setup Gate

The application-level frontend guard that runs before authentication providers,
probes public `/api/setup/status`, and chooses setup, normal application, or an
explicit probe-failure state.

## Same-Origin API

The browser API topology in which production pages and `/api/*` are served by
the same Taco executable, while development uses a Next.js proxy. Taco has no
backend CORS support under this model.

## Data-Directory-Owned Avatar Storage

The fixed local avatar path `<data-dir>/uploads/avatars`, derived from Taco's
explicit installation data directory so mutable upload data stays on the
durable volume.

## Minimal Connection Field Defaults

The setup-form defaults of PostgreSQL port `5432` and Redis port `6379`, plus
the rule that empty optional Redis account, password, or database fields are
omitted so Redis uses its own defaults.

## Simple TLS Setup Control

The independently enabled-by-default PostgreSQL or Redis setup switch that
maps to strict TLS certificate and hostname verification when enabled, or an
explicit plaintext transport choice when disabled.

## Structured Infrastructure Setup Inputs

The setup-form contract that collects PostgreSQL and Redis host, port,
account, password, and database values independently instead of accepting a
single raw connection URL.

## Stateless Setup Test

A PostgreSQL or Redis setup connection test that uses only request-supplied
values and writes no installation configuration. Final installation repeats
all dependency validation before destructively resetting the selected data
stores; installation-state persistence remains atomic.

## Public Setup Route Access

The first-installation access model in which all setup-mode API operations are
public. Taco has no setup token, setup credential field, or setup-authentication
request header; setup mutations disappear after installation completes.

## Atomic Installation State File

The single encrypted file containing Taco's installation configuration and
completion state. It is durably written through a temporary file and atomic
replacement; only absence means setup mode.

## Authenticated Installation Configuration Encryption

The versioned `XChaCha20-Poly1305` envelope used for persistent Taco
installation configuration. It uses an operator-held 32-byte Base64URL root
key and a fresh 24-byte nonce per write, detecting both disclosure and
tampering.

## Bootstrap Listener

The HTTP address available before installation configuration exists. Taco
defaults to `0.0.0.0:3000` and accepts one explicit override through `--listen`
or `TACO_LISTEN_ADDR`.

## Installation Data Directory

The explicit persistent root supplied through exactly one of `--data-dir` or
`TACO_DATA_DIR`. It contains encrypted installation state and uploads, and is
distinct from immutable frontend static artifacts.

## Destructive Setup Reset

The setup-mode installation sequence that, after validating the submitted
connections, recreates the selected PostgreSQL `public` schema and executes
`FLUSHALL` against Redis before migrations and fresh owner creation.

## Installation Super-Administrator Protection

The rule that normal user management cannot disable, delete, demote, or
replace the installation super administrator. Its profile and password are
self-service; a new owner exists only after destructive setup reset.

## Marker-Based Super-Administrator Authorization

The authorization model in which only the unique installation-super-
administrator account marker grants global permission and data-scope bypass.
All business accounts receive authority solely through explicitly assigned RBAC
roles.

## Business Administrator

An operational account created after installation through normal user and role
management. Taco does not preseed a business-administrator role; its
permissions are explicitly assigned through RBAC and it is never an
installation recovery target.

## Installation Super Administrator

The single account identified by Taco's installation ownership marker. It is
created only in the web setup flow after destructive setup reset.

## Installation Reset

An operator-local command that removes Taco's encrypted installation
configuration and completion state so the next start enters setup mode. It is
run while the backend is stopped as `taco installation reset --confirm-reset`
and does not need the former encryption root key. The next setup submission
resets the selected PostgreSQL schema and Redis instance before installation.

## Immutable Infrastructure Configuration

The policy that PostgreSQL, Redis, JWT, and other startup secrets are set at
installation and cannot be changed through the running Taco administration UI.
They remain in encrypted installation configuration rather than `sys_config`.

## Minimal Three-Step Setup

The first-visit installation wizard consisting only of PostgreSQL connection,
Redis connection, and initial-administrator steps. JWT is generated by Taco;
other required startup values come from an explicit persisted installation
profile. The final confirmation page can expand validated advanced overrides;
leaving them collapsed explicitly selects the profile values.

## Initial Administrator

The first Taco super administrator created as a required setup-mode step after
initial migrations. Its account is created through the existing `user`
installation-owner use case and its password exists only as a database hash.

## Automatic Initial Migrations

The one-time setup-mode action that resets validated PostgreSQL and Redis
targets, then applies all database migrations. It is distinct from normal
runtime, which rejects a schema with pending migrations.

## Installation Restart

The transition after successful installation in which the setup-mode process
exits gracefully and the deployment supervisor restarts it into normal runtime
mode using the persisted installation configuration.

## Single-Replica Installation

The deployment constraint that exactly one Taco backend instance runs while
initial installation is incomplete. It avoids requiring distributed setup
locking, migration coordination, and restart orchestration.

## Configuration Encryption Root Key

The long-lived operator-controlled secret used to encrypt and decrypt Taco's
persistent installation configuration. It is generated before startup and is
not stored beside the encrypted configuration. It is required and validated in
both setup mode and normal runtime.

## Secret Generation Workbench

The local operator command that generates a configuration encryption root key
before a Taco deployment is started. It prints only a copyable key-value line
and never writes the key into Taco's installation data directory.

## Frontend Delivery Mode

The deployment choice between a standalone Next.js development server and a
production static-export bundle embedded in the Rust release executable, both
built from the same Taco frontend source.

## Public First Installation

The policy that the first visitor able to reach an uninstalled Taco deployment
may complete installation. The configuration-encryption root key remains a
server-side startup secret and is not used as a browser credential.

## Setup Mode

The restricted backend runtime used only before Taco has completed its first
installation. It exposes the installation flow without constructing the normal
database- and Redis-dependent application.

## Installation Configuration

The persistent, deployment-managed configuration written by the first-visit
installation flow. It contains startup infrastructure settings and completion
state, survives process or container restart, and is distinct from the
non-sensitive runtime parameters in `sys_config`.

## System Log

A persisted application runtime event emitted through `taco_tracing` that passes the configured `sys.observability.tracingConfig.log_level` filter. It is distinct from operation logs and login logs, which are business audit records.

## Rolling Retention

A retention policy that keeps records newer than a moving cutoff. For the initial system-log policy, the cutoff is seven days before the cleanup task execution time.

## Cleanup Batch

One bounded deletion operation performed by the system-log cleanup task. Its maximum record count is a configurable operational parameter.

## Runtime Tracing Level

The `sys_config` value that controls which `taco_tracing` events are admitted to the reloadable tracing subscriber. A valid persisted value is required for application startup and can be changed without a restart.

## Cluster-Wide Reload

The process by which every backend instance reloads the persisted runtime tracing configuration after a committed PostgreSQL notification. The listener subscribes before its database snapshot and reconnects with a mandatory reconciliation read after failure.

## System Log Full-Text Search

PostgreSQL indexed keyword search across the event message and the values of its structured fields.

## System Log Cleanup Schedule

The scheduler-owned cron expression that determines when the enabled system-log cleanup task executes. It is independent from the rolling retention duration.

## System Log Cleanup Parameters

The scheduler task parameters `retention_days` and `batch_size`, which define the rolling expiration cutoff and the maximum rows deleted by one database operation.

## System Log Cleanup Batch Bound

The inclusive allowed range of `1..10000` records for a single cleanup deletion operation; the default is `1000` records.

## System Log Cleanup Completion

The task behavior that repeats independently committed cleanup batches until no records remain older than the rolling retention cutoff.

## System Log Ingestion Overload Policy

The explicit behavior that preserves business availability by discarding logs when the bounded persistence queue is full or PostgreSQL writes fail, while exposing loss and failure telemetry.

## System Log Record

A persisted tracing event with fixed timestamp, level, Rust source-module-path target, and message columns, plus the complete structured event fields stored as `JSONB`. Admission is distinct from the target field.

## System Log Query Contract

The indexed list-query interface with keyword, level, time-range, and target filters; it defaults to the last 24 hours and uses descending `(occurred_at, id)` cursor pagination.

## System Log Administrative Actions

Privileged operations to delete one or more system logs, manually clear logs, and export a time-bounded result set.

## Filtered System Log Manual Cleanup

A privileged bulk deletion that removes only records matching the active filters and a required time range, after showing the matching record count.

## System Log Export

An XLSX artifact for a required time range containing each record's fixed fields and complete structured-fields JSON document. Long messages and fields JSON use ordered continuation worksheets keyed by log ID so export remains lossless within Excel cell and worksheet limits.

## Default System Log Cleanup Schedule

The initial enabled scheduler cron `0 0 19 * * *`, which executes at 03:00 China Standard Time because the scheduler uses UTC.

## System Log RBAC

The established log-module permission pattern with separate `list`, `query`, `remove`, and `export` capabilities, seeded through the existing menu and role assignment mechanism.

## Centralized Sensitive Data Redaction

The dependency-free `kernel::redaction` policy that recursively masks values for sensitive field names before audit snapshots or tracing events reach standard output or persistence.

## Runtime-Configurable HTTP Log Capture

Cluster-wide runtime settings that independently enable or disable HTTP access logs and capture of request bodies, response bodies, URL query parameters, and request headers; redaction is always enforced.

## Default HTTP Log Capture

The runtime default that enables HTTP access summaries and sanitized query parameters while disabling request and response bodies and request headers.

## Nonblocking System Log Ingestion

The custom tracing layer, bounded queue, background PostgreSQL batch writer, and stateful HTTP tee middleware that persist logs without awaiting database I/O on business request paths.

## HTTP Body Capture Limit

The runtime-configurable per-request and per-response body-copy bound, defaulting to `16 KiB` and valid from `0` through `64 KiB`; any system-log event is still limited to `128 KiB` in total.

## Daily System Log Partition

A UTC calendar-day PostgreSQL range partition created by the asynchronous persistence worker before it inserts system-log records for that day.

## Required System Log Cleanup Task

The default cleanup scheduler task whose cron and validated parameters are editable but whose enabled state and existence are enforced by scheduler task-definition lifecycle metadata.

## Multilingual System Log Search

The combined PostgreSQL full-text and `pg_trgm` trigram index strategy for English words, Chinese text, identifiers, and arbitrary substrings.

## HTTP Log Level Filtering

The policy that emits HTTP access summaries at `INFO` and admits them only when both HTTP access logging is enabled and the global tracing level permits `INFO` events.

## Runtime Tracing Configuration

The non-public `sys.observability.tracingConfig` JSON parameter that owns tracing level, HTTP capture settings, and slow-operation thresholds; its seed has a descriptive name and remark.

## No Local Log Migration

The decision to begin PostgreSQL system-log persistence at deployment without importing historical local formatted tracing files.

## HTTP Log Route Exclusions

The default exclusion of health, metrics, documentation, OpenAPI, and static or uploaded-file routes from HTTP access system-log capture.

## System Log Ingestion Queue Boundaries

The fixed asynchronous queue capacity of 512 events, with PostgreSQL writer batches of at most 100 events and a maximum incomplete-batch wait of 100 milliseconds.

## System Log Cleanup Execution Policy

The scheduler policy that prevents concurrent cleanup executions and performs one recovery run for missed cron occurrences.

## Complete System Log Levels

The full `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR` severity set supported by tracing helpers, runtime configuration, system-log queries, and the toolbar.

## Observability Bounded Context

The `crates/observability` bounded context that owns system-log records, persistence, management APIs, export, and retention cleanup while tracing and scheduling depend only on its ports.

## Infrastructure Call Log Scope

The policy that logs only failed or slow PostgreSQL, Redis, and outbound HTTP operations, while Prometheus records aggregate normal-path telemetry.

## Slow Infrastructure Operation Thresholds

Positive, runtime-configurable latency thresholds: `500 ms` for PostgreSQL, `100 ms` for Redis, and `1000 ms` for outbound HTTP.

## Local Tracing File Removal

The replacement of the daily local tracing file appender with PostgreSQL persistence while retaining standard-output tracing.
