# ADR 0016: Default System Log Cleanup Schedule

- Status: Accepted
- Date: 2026-07-16

## Context

The cleanup task must be imported and enabled on deployment, so it needs an unambiguous initial execution schedule.

## Decision

Seed the enabled cleanup task with the UTC cron expression `0 0 19 * * *`. The scheduler interprets it as 19:00 UTC, which is 03:00 China Standard Time on the following calendar day. Administrators can later update the scheduler-owned cron expression.

## Consequences

- The first cleanup run occurs daily at the agreed local off-peak time.
- The persisted scheduler expression explicitly reflects the scheduler's UTC time basis.
- Retention remains controlled separately by the task's `retention_days` parameter.
