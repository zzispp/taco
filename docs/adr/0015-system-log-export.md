# ADR 0015: System Log Export

- Status: Accepted
- Date: 2026-07-16

## Context

Operators need to take a portable, time-bounded copy of system logs for incident investigation.

## Decision

Export system logs as XLSX using the established audit-log export mechanism. Each export requires a time range and includes timestamp, level, target module, message, and the complete structured-fields JSON document. Values that exceed Excel cell limits use the lossless continuation-sheet contract in ADR 0036. Export pagination uses the existing global export batch configuration.

## Consequences

- System-log exports use a consistent artifact format with other audit modules.
- No tracing context is lost when it is not represented by a fixed column.
- The API validates a time range before beginning the export stream.
