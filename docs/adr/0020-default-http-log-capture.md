# ADR 0020: Default HTTP Log Capture

- Status: Accepted
- Date: 2026-07-16

## Context

HTTP diagnostic capture must be useful by default without continuously retaining high-volume bodies and headers.

## Decision

Enable HTTP access-summary logging and URL query-parameter capture by default. Disable request-body, response-body, and request-header capture by default. Operators can change each setting through the cluster-wide runtime tracing configuration.

## Consequences

- Routine operation records request outcomes and sanitized query context at low overhead.
- Higher-cost body and header capture is explicitly enabled only when needed for diagnosis.
- All enabled HTTP fields remain subject to centralized redaction and capture-size limits.
