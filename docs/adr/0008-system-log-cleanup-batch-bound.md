# ADR 0008: System Log Cleanup Batch Bound

- Status: Accepted
- Date: 2026-07-16

## Context

Cleanup must limit the amount of work in each database deletion while allowing administrators to tune the task for their deployment.

## Decision

The cleanup task parameter `batch_size` defaults to `1000`. Valid values range from `1` through `10000`, inclusive. The value limits one database deletion operation.

## Consequences

- Each deletion transaction has a bounded row count.
- Administrators can tune cleanup throughput within an explicit operational range.
- The task execution contract must define whether one invocation deletes one batch or continues until no expired records remain.
