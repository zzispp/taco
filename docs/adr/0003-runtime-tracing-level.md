# ADR 0003: Runtime Tracing Level

- Status: Accepted
- Date: 2026-07-16

## Context

Operators must be able to change the application tracing level online. A startup-only YAML value requires a process restart and does not meet that operational need.

## Decision

Store the tracing level in `sys_config` as the sole runtime source of truth. The application initializes a reloadable tracing filter from the persisted value, and a successful parameter update reloads the filter without restarting the process.

The schema migration must seed a valid value before the runtime reads it. A missing or invalid persisted value prevents application startup and is not replaced with a YAML or code default.

## Consequences

- The tracing level is managed through the existing parameter-management UI and API.
- A parameter update must validate the level before persistence and must notify the running tracing subscriber.
- Startup order must make the migrated `sys_config` value available before tracing initialization.
