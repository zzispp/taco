# ADR 0010: System Log Ingestion Overload Policy

- Status: Accepted
- Date: 2026-07-16

## Context

Tracing events are emitted on business execution paths. PostgreSQL may be temporarily unavailable or unable to keep up with the ingestion rate.

## Decision

Prioritize business availability. System logs are delivered through bounded asynchronous batching. When the queue is full or PostgreSQL writes fail, the affected system-log events are explicitly discarded rather than blocking business requests.

The runtime exposes an ingestion-drop counter and the most recent write-failure state for monitoring and alerting. It must not silently discard events.

## Consequences

- A logging database incident does not make request handling unavailable.
- System-log completeness is not guaranteed during overload or persistence failure.
- Operators have observable evidence of lost logs and ingestion failures.
