# ADR 0028: HTTP Log Level Filtering

- Status: Accepted
- Date: 2026-07-16

## Context

HTTP access logging has its own capture switch, but system-log admission must remain coherent with the tracing pipeline's global severity filter.

## Decision

Emit HTTP access summaries at `INFO` severity. The HTTP access switch determines whether the middleware emits an access event; `sys.observability.tracingConfig.log_level` determines whether that event is admitted to system-log persistence. Raising the global level to `WARN` or `ERROR` suppresses HTTP access summaries with other lower-severity tracing events.

## Consequences

- One global severity setting consistently controls system-log volume.
- Operators can disable access logging independently or suppress it together with all lower-severity events.
- The HTTP middleware must apply its capture settings before emitting the event through the tracing pipeline.
