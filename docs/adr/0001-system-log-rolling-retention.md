# ADR 0001: System Log Rolling Retention

- Status: Accepted
- Date: 2026-07-16

## Context

System logs will be persisted in PostgreSQL and require automatic retention management.

## Decision

Retain system log records for seven days on a rolling basis. The cleanup job deletes only records whose timestamp is older than the retention cutoff calculated from the current execution time.

The seven-day retention period does not mean clearing all logs every seven days. Cleanup must operate in configured batches so a large deletion does not become a single database operation.

## Consequences

- Recent system logs remain continuously available for queries and investigation.
- Storage use is bounded by the configured retention period and log volume.
- The cleanup task needs a cutoff timestamp, a deterministic ordering, and a configured per-batch deletion limit.
