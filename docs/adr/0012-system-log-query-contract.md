# ADR 0012: System Log Query Contract

- Status: Accepted
- Date: 2026-07-16

## Context

System-log browsing must support incident investigation without requiring expensive offset pagination or unbounded sorting.

## Decision

The system-log toolbar provides keyword search, multi-select severity levels, a time range, and target-module filtering. It defaults to the last 24 hours and allows a range within the retained data window. Results use descending `(occurred_at, id)` cursor pagination.

## Consequences

- Queries can use composite timestamp-level-target indexes and the full-text-search index.
- The default view favors recent operational events.
- Pagination remains stable and performant as retained log volume grows.
