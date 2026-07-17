# ADR 0025: Daily System Log Partitioning

- Status: Accepted
- Date: 2026-07-16

## Context

The expected system-log volume is unknown, but the design must favor high-volume ingestion and time-bounded investigation. The retention window and query contract are inherently time-based.

## Decision

Partition system logs by UTC calendar day. The asynchronous persistence worker ensures the target day's partition exists before inserting its batch; HTTP and business request paths never perform partition DDL. Time-range queries rely on PostgreSQL partition pruning.

The scheduled retention task continues to delete expired records in the agreed independent batches rather than changing its contract to unconditional partition drops.

## Consequences

- Write and query performance scales across short-lived daily partitions.
- Partition availability is independent of the administrator-configurable cleanup cron.
- Schema management must create indexes for every new partition and test date-boundary behavior.
