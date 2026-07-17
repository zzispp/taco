# ADR 0034: Complete System Log Levels

- Status: Accepted
- Date: 2026-07-16

## Context

The system-log toolbar and runtime tracing level must represent the full standard tracing severity model. The current helper macros only emit `INFO`, `WARN`, and `ERROR` events.

## Decision

Support `TRACE`, `DEBUG`, `INFO`, `WARN`, and `ERROR` throughout system-log capture, query filtering, and presentation. Extend the internal tracing helpers with structured `TRACE` and `DEBUG` emitters. Runtime `log_level` accepts only these five levels.

## Consequences

- Operators can select and investigate every standard tracing severity.
- Raising the global logging level has predictable effect across all emitted severities.
- The frontend uses one exhaustive, typed level enumeration.
