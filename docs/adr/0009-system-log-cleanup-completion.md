# ADR 0009: System Log Cleanup Completion

- Status: Accepted
- Date: 2026-07-16

## Context

The per-operation batch bound prevents large deletion transactions, but deleting only one batch per task execution can leave expired data permanently accumulated.

## Decision

One cleanup task execution repeatedly deletes expired records until none remain. Each batch uses an independent transaction and deletes no more than `batch_size` records. The execution result records both the total deleted record count and the number of completed batches.

## Consequences

- The rolling retention cutoff is fully enforced after a successful task execution.
- No deletion transaction exceeds the configured batch bound.
- Scheduler execution details expose cleanup throughput and progress.
