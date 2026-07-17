# ADR 0038: Reconciled Cluster Tracing Reload

- Status: Accepted
- Date: 2026-07-17

## Context

PostgreSQL `NOTIFY` is connection-scoped and transient. Reading the tracing
configuration before subscribing can miss a committed update; returning from a
failed listener leaves an instance permanently stale.

## Decision

Treat the persisted tracing configuration as the sole source of truth and
`NOTIFY` only as a reload trigger. Each process subscribes before reading its
initial database snapshot. On a listener failure it records an unhealthy state,
reconnects and re-subscribes, then immediately re-reads and atomically applies
the latest valid configuration.

## Consequences

- A transient notification gap cannot leave an instance permanently stale.
- During a database outage the process continues with its last valid runtime
  configuration while listener health remains observable.
- Reload tests must cover startup races and listener disconnect/reconnect.
