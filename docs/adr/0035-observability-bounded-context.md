# ADR 0035: Observability Bounded Context

- Status: Accepted
- Date: 2026-07-16

## Context

System logs now include high-throughput event persistence, runtime configuration, query and export APIs, retention, and administrative actions. These responsibilities do not belong to business auditing, generic tracing helpers, or general system administration.

## Decision

Create `crates/observability` as the system-log bounded context. It owns the system-log domain model, PostgreSQL partition persistence, query/delete/export API, and retention cleanup use case. `crates/tracing` owns event capture, redaction invocation, and nonblocking emission interfaces. `crates/scheduler` triggers cleanup only through a port. `apps/backend` composes concrete implementations and routes.

## Consequences

- System-log management stays independent from operation and login audit business rules.
- Tracing does not depend on PostgreSQL querying or HTTP administration APIs.
- Scheduler does not depend on observability infrastructure implementations.
- Dependency direction remains compatible with DDD and Clean Architecture.
