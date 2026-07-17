# ADR 0032: System Log Ingestion Queue Boundaries

- Status: Accepted
- Date: 2026-07-16

## Context

The asynchronous persistence queue needs a fixed memory bound and a write cadence that preserves low latency without allowing arbitrary runtime reconfiguration of an active channel.

## Decision

Set the internal queue capacity to `512` events. The writer flushes up to `100` events per batch and waits at most `100 ms` for an incomplete batch. These values are fixed internal boundaries, not runtime parameters.

## Consequences

- At the `128 KiB` maximum event size, queued raw event payload is bounded to approximately `64 MiB`.
- Queue saturation follows the explicit observable-drop policy.
- Runtime tracing configuration does not attempt to resize an active asynchronous channel.
