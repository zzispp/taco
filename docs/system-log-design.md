# System Log Design

## Scope

Replace local rolling tracing files with PostgreSQL-backed system logs. Keep tracing on standard output. Do not migrate historical local files.

`crates/observability` owns log records, PostgreSQL persistence, management APIs, exports, and retention cleanup. `crates/tracing` owns capture and nonblocking event emission. The backend composition root wires these capabilities together.

## Capture And Safety

- Persist `taco_tracing` events that pass the runtime severity filter.
- Support `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR`.
- Persist fixed timestamp, level, target module path, and message columns plus complete JSONB fields. System-log admission uses a separate internal marker.
- Redact sensitive fields through `kernel::redaction` before events reach standard output or the persistence queue.
- Keep standard output; remove the local file appender and `tracing.file` settings.
- Use a bounded asynchronous queue. Event emission never awaits PostgreSQL.
- Queue: 512 events; writer: 100 events or 100 ms; total serialized event limit: 128 KiB.
- Queue saturation, write failures, and oversize records are observable drops.

## HTTP And Infrastructure

- Add a stateful Axum middleware for HTTP access logs and optional bounded request/response body capture.
- Exclude health, metrics, docs, OpenAPI, static, and upload routes.
- Default HTTP capture: access summary and query parameters enabled; request body, response body, and headers disabled.
- HTTP options reload at runtime; sensitive values remain redacted in every mode.
- HTTP body limit: default 16 KiB, configurable from 0 through 64 KiB.
- HTTP access events are `INFO` and obey the global tracing level.
- Log PostgreSQL, Redis, and outbound HTTP failures plus slow calls only.
- Slow thresholds: PostgreSQL 500 ms, Redis 100 ms, outbound HTTP 1000 ms.

## Runtime Configuration

Use non-public `sys.observability.tracingConfig` with a seed name and remark:

```json
{
  "log_level": "info",
  "http": {
    "access_enabled": true,
    "capture_request_body": false,
    "capture_response_body": false,
    "capture_query_parameters": true,
    "capture_request_headers": false,
    "max_body_capture_bytes": 16384
  },
  "slow_operation_ms": {
    "postgres": 500,
    "redis": 100,
    "outbound_http": 1000
  }
}
```

Update it cluster-wide with PostgreSQL `LISTEN/NOTIFY`; the persisted value is the source of truth. Each process subscribes before reading its startup snapshot, reconnects and reconciles after listener failure, then atomically reloads the tracing filter and HTTP capture settings. Missing or invalid configuration fails startup.

## Persistence And Querying

- Store records in UTC daily PostgreSQL partitions; the asynchronous writer ensures the target partition exists.
- Query with cursor pagination ordered by descending `(occurred_at, id)`.
- Toolbar: keyword, severity multi-select, time range, and target module; default range is the last 24 hours.
- Index full-text content and `pg_trgm` trigrams to support English words, Chinese text, identifiers, and substrings.
- Full-text search includes message and structured field values.

## Administration

- Add System Logs after Login Logs at `/dashboard/monitor/logs/system-logs`.
- Use separate `list`, `query`, `remove`, and `export` permissions, seeded like the existing audit-log menus.
- Support detail, single deletion, batch deletion, filtered manual cleanup, and XLSX export.
- Manual cleanup and export require a time range. Manual cleanup confirms the matched-record count before deleting.
- Export fixed columns plus complete fields JSON. XLSX exports are lossless: values exceeding an Excel cell limit use ordered continuation worksheets keyed by log ID and value kind; additional worksheets are created before the Excel row limit.

## Retention Job

- Seed and enable the system-log cleanup job.
- Default cron: `0 0 19 * * *` UTC, equivalent to 03:00 China Standard Time.
- Parameters: `retention_days: 7` and `batch_size: 1000`; permitted batch range is 1 through 10000.
- One execution deletes independent batches until no records are older than the cutoff; execution details report total records and batches.
- The job is non-concurrent, runs one missed-occurrence recovery, and cannot be disabled or deleted.
- Administrators can change its cron and validated cleanup parameters in scheduler management.
