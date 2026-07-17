# ADR 0007: System Log Cleanup Task Parameters

- Status: Accepted
- Date: 2026-07-16

## Context

The system-log cleanup behavior needs a rolling retention duration and a bounded deletion size. These settings describe cleanup execution rather than event recording.

## Decision

The default imported system-log cleanup task owns the `retention_days` and `batch_size` parameters. Administrators manage them with the task cron in scheduler management.

## Consequences

- The task validates its parameters before it can be saved or executed.
- Task configuration and execution history provide an audit trail for retention-policy changes.
- Parameter management owns event-recording settings such as tracing level, but not cleanup behavior.
