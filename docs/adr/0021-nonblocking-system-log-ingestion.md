# ADR 0021: Nonblocking System Log Ingestion

- Status: Accepted
- Date: 2026-07-16

## Context

System logs must capture tracing and optional HTTP context without adding database latency to business requests.

## Decision

Use a custom `tracing_subscriber::Layer` that serializes and redacts events, then non-blockingly sends them to a bounded asynchronous queue. A background worker batches PostgreSQL writes. HTTP capture uses an Axum stateful middleware that conditionally tees request and response bodies into the same queue without waiting for persistence.

The runtime configuration owns `http.max_body_capture_bytes`, defaulting to `16 KiB` with an allowed range of `0..64 KiB`. Every persisted event has a fixed, non-configurable maximum serialized size of `128 KiB`.

## Consequences

- Business request handling never awaits system-log persistence.
- The capture depth can be tuned online, while the hard event bound protects queue, database, full-text index, and export capacity.
- Queue saturation, write failures, and oversize events are surfaced through explicit telemetry.
- The existing tracing reload handle and PostgreSQL notification pattern support cluster-wide configuration changes.
