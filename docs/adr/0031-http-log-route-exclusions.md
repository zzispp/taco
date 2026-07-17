# ADR 0031: HTTP Log Route Exclusions

- Status: Accepted
- Date: 2026-07-16

## Context

Some HTTP routes are scraped, browsed, or served at high frequency without representing business operations. Logging them would obscure diagnostic signals and add avoidable persistence volume.

## Decision

Exclude `/health`, `/metrics`, `/docs*`, `/openapi.json`, and static or uploaded-file routes from HTTP access system logs. Continue logging all business API routes, including system-log management APIs.

## Consequences

- Monitoring and documentation traffic does not flood system-log storage.
- System-log management actions remain observable like other business APIs.
- The HTTP middleware applies route exclusion before body capture and event construction.
