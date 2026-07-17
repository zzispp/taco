# ADR 0002: System Log Capture Scope

- Status: Accepted
- Date: 2026-07-16

## Context

The local tracing file sink will be replaced by PostgreSQL-backed system-log storage. The system needs a precise boundary for persisted events.

## Decision

Persist every event emitted through `taco_tracing` that passes the configured `sys.observability.tracingConfig.log_level` filter. Do not persist events emitted by third-party dependencies.

## Consequences

- Persisted records retain the diagnostic scope of the current application log stream.
- The configured level determines which application events can be queried later.
- Third-party library verbosity cannot unexpectedly increase system-log storage volume.
