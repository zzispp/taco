# ADR 0033: System Log Cleanup Execution Policy

- Status: Accepted
- Date: 2026-07-16

## Context

Cleanup can be delayed by deployment or downtime, and concurrent cleanup executions would contend for the same expired records.

## Decision

Configure the required system-log cleanup task as non-concurrent. If its cron occurrence is missed, the scheduler performs one recovery execution rather than replaying every missed occurrence.

## Consequences

- Only one cleanup execution processes expired records at a time.
- Downtime is recovered by one complete batch-loop cleanup run.
- Retention recovery does not multiply database work by replaying obsolete cron occurrences.
