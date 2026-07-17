# Glossary

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
