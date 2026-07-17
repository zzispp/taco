# ADR 0024: Remove Local Tracing Files

- Status: Accepted
- Date: 2026-07-16

## Context

Tracing currently writes both to standard output and to daily rolling local files. System logs are moving to PostgreSQL for managed querying and retention.

## Decision

Remove the local rolling file appender and all `tracing.file` configuration. Keep standard-output tracing while PostgreSQL receives persisted system logs.

## Consequences

- The application no longer creates or retains local tracing files.
- Container and host log collectors can still consume immediate standard output.
- PostgreSQL is the managed source for system-log browsing, deletion, export, and retention.
