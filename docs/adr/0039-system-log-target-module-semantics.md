# ADR 0039: System Log Target Module Semantics

- Status: Accepted
- Date: 2026-07-17

## Context

The persisted tracing target was a fixed admission label, so every system-log
record had the same target and target filtering could not identify a source.

## Decision

Persist the emitting Rust module path as the system-log `target`, for example
`user::api::handlers::auth`. System-log admission uses a separate internal
marker and never overloads the queryable target column. The toolbar accepts
target text for prefix or keyword filtering.

## Consequences

- Operators can locate records by bounded-context and source-module path.
- Emission helpers do not need manually maintained source labels.
- The persistence layer must validate the admission marker independently from
  the target value.
