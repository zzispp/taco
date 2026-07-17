# ADR 0037: Redact Tracing Before All Sinks

- Status: Accepted
- Date: 2026-07-17

## Context

System logs are persisted asynchronously, while the same tracing events also
remain on standard output for container and platform diagnostics. Redacting
only the persistence-layer copy permits a structured event to expose sensitive
values through standard output.

## Decision

Apply the centralized tracing redaction policy before an event reaches any
tracing layer. Standard output and PostgreSQL system-log persistence therefore
observe the same redacted message and fields. Context-specific HTTP URL and
body rules remain the responsibility of their capture context before event
emission.

## Consequences

- Sensitive values cannot enter standard output through `taco_tracing` events.
- Macros and helpers must construct safe event fields before invoking `tracing`.
- Redaction tests must cover both the persistence layer and formatted standard
  output.
