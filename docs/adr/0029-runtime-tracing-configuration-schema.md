# ADR 0029: Runtime Tracing Configuration Schema

- Status: Accepted
- Date: 2026-07-16

## Context

Tracing level, HTTP capture depth, and slow-operation thresholds are related runtime observability controls that must update cluster-wide from one persisted source.

## Decision

Store the non-public runtime configuration under `sys.observability.tracingConfig` with this validated JSON shape:

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

The migration seed includes a clear parameter name and `remark` description, following the established system-configuration convention. The fixed `128 KiB` total event bound is intentionally excluded from runtime configuration.

## Consequences

- Operators edit one documented parameter instead of multiple related keys.
- Backend parsing rejects unknown or invalid fields before applying a cluster-wide reload.
- The parameter is not exposed through the public application-configuration endpoint.
