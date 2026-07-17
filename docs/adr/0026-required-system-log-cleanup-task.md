# ADR 0026: Required System Log Cleanup Task

- Status: Accepted
- Date: 2026-07-16

## Context

System-log retention is a required operational control. Allowing an administrator to stop the cleanup task would leave persisted runtime logs without bounded retention.

## Decision

The default system-log cleanup task is required to remain enabled. Administrators can change its cron expression and validated cleanup parameters but cannot disable or delete it.

Task lifecycle policy belongs to `ScheduledTaskDefinition` metadata rather than the execution-only `ScheduledTask` trait. Every registered task declares a lifecycle policy; scheduler application services enforce it on status changes, and the API exposes it for the UI.

## Consequences

- The backend rejects attempts to stop or delete the required cleanup task, independent of UI behavior.
- Existing tasks explicitly retain their normal administrable lifecycle policy.
- The lifecycle policy applies to both status changes and destructive task actions.
