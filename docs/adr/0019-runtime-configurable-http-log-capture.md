# ADR 0019: Runtime-Configurable HTTP Log Capture

- Status: Accepted
- Date: 2026-07-16

## Context

HTTP request and response context is needed for diagnosis, but its payload cost and sensitivity vary by deployment and incident.

## Decision

Extend the runtime tracing configuration with independently configurable HTTP-capture switches for access logging, request bodies, response bodies, URL query parameters, and request headers. The configuration is managed in parameter management and reloads cluster-wide through the existing tracing configuration notification path.

Centralized sensitive-data redaction remains mandatory for every enabled capture mode and cannot be disabled by runtime configuration.

## Consequences

- Operators can increase or reduce HTTP diagnostic detail without restarting services.
- The tracing runtime must atomically apply both filter-level and HTTP-capture configuration updates.
- Body-size and total-event-size limits remain separate requirements to protect performance.
