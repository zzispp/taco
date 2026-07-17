# ADR 0013: System Log Administrative Actions

- Status: Accepted
- Date: 2026-07-16

## Context

Administrators need to remove selected system logs, clear records deliberately, and export investigation data in addition to searching and viewing records.

## Decision

The system-log module provides single-record deletion, batch deletion, manual cleanup, and export. Every export requires an explicit time range. The frontend reuses the established audit-log toolbar time-range component and interaction pattern.

## Consequences

- The API and RBAC catalog need distinct permissions for destructive actions and export.
- Manual cleanup semantics need a precise filter and confirmation contract.
- Export volume is bounded by an operator-selected time range.
