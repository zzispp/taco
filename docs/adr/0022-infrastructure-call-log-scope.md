# ADR 0022: Infrastructure Call Log Scope

- Status: Accepted
- Date: 2026-07-16

## Context

The project uses PostgreSQL, Redis, and outbound HTTP. Persisting every dependency operation as a system log would create excessive write volume and can expose query parameters or cached values.

## Decision

Persist infrastructure call failures and calls exceeding their configured slow-operation threshold. Do not persist every successful PostgreSQL, Redis, or outbound HTTP call. Continue using Prometheus metrics for aggregate normal-path throughput and latency.

## Consequences

- System logs retain actionable dependency failures and latency anomalies.
- Normal-path observability remains low-cardinality and low-overhead.
- Slow-operation thresholds need a validated runtime configuration contract.
