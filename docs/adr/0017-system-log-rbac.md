# ADR 0017: System Log RBAC

- Status: Accepted
- Date: 2026-07-16

## Context

The existing login-log and operation-log modules establish the repository's menu and permission-seeding convention.

## Decision

The system-log module follows the existing audit-log RBAC pattern. Its menu is placed after Login Logs and has separate `list`, `query`, `remove`, and `export` permissions. Seed role assignments follow the same default role-grant policy as the existing log modules.

## Consequences

- Menu visibility and API authorization remain consistent with established log management.
- Viewing a detail, deleting records, and exporting records remain independently grantable capabilities.
- No special role or parallel authorization mechanism is introduced.
