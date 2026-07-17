# ADR 0023: Slow Infrastructure Operation Thresholds

- Status: Accepted
- Date: 2026-07-16

## Context

Different dependency types have different expected latency profiles. Slow-call logging needs operationally adjustable thresholds to avoid noise.

## Decision

Store positive, cluster-reloadable slow-operation thresholds in the runtime tracing configuration. Default thresholds are `500 ms` for PostgreSQL, `100 ms` for Redis, and `1000 ms` for outbound HTTP.

## Consequences

- Operators can tune anomaly logging without application restarts.
- PostgreSQL, Redis, and outbound HTTP retain independent latency baselines.
- The slow-operation instrumentation must record only safe operation metadata, never SQL parameters, cache values, or sensitive request material.
