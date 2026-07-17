# ADR 0011: System Log Record Shape

- Status: Accepted
- Date: 2026-07-16

## Context

Operators need efficient list queries while investigations need access to all event context. Future tracing fields must not require a schema migration for every addition.

## Decision

Each system-log record stores fixed query columns for timestamp, severity level, target module, and message, plus a complete `JSONB` document containing structured event fields. The target is the automatically captured Rust module path, while admission is represented separately (ADR 0039). The list view uses fixed columns; the detail view exposes the complete field document.

## Consequences

- PostgreSQL indexes can optimize common list filters without parsing JSON per row.
- Full-text search can include both the message and field values.
- New tracing fields can be retained without a relational schema change.
