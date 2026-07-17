# ADR 0006: Configurable System Log Cleanup Schedule

- Status: Accepted
- Date: 2026-07-16

## Context

The system-log retention task must be enabled by default, while administrators need to change its execution frequency online.

## Decision

Manage the cleanup frequency in the scheduler job. The initial seven-day rolling retention policy remains independent from the task execution frequency.

## Consequences

- The configured cron expression is validated by the scheduler before it is accepted.
- Parameter management does not own or duplicate the cleanup schedule.
- The default imported task must respond to a runtime schedule change without a process restart.
