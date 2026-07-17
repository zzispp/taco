# ADR 0004: Cluster-Wide Tracing Reload

- Status: Accepted
- Date: 2026-07-16

## Context

The tracing level is a runtime parameter and the backend can run more than one process. An in-process-only reload would leave instances on different levels after an online update.

## Decision

Every running backend instance must apply a committed tracing-level update. The configuration write publishes a PostgreSQL `NOTIFY` event after commit. Each backend instance subscribes before reading its initial persisted value, then treats notifications as reload triggers. It re-reads, validates, and atomically applies the database value; listener failures reconnect, re-subscribe, and reconcile the latest value (ADR 0038).

## Consequences

- A tracing-level update has cluster-wide behavior rather than process-local behavior.
- The notification mechanism is a wake-up signal, not the source of configuration truth.
- Each instance owns a dedicated PostgreSQL listener connection and its tracing reload lifecycle.
