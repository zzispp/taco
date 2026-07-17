# ADR 0005: System Log Full-Text Search

- Status: Accepted
- Date: 2026-07-16

## Context

System-log investigation requires keyword search beyond the human-readable message. Operational identifiers are commonly emitted as event fields.

## Decision

Keyword search covers both the event message and structured field values. PostgreSQL full-text search indexes the combined searchable content; queries must use that index rather than scanning raw log rows.

## Consequences

- Event fields must be persisted in a structured form as well as being available to the search document.
- Operators can locate records by values such as request, user, or job identifiers when those fields are present.
- The system-log schema needs a maintained full-text-search document and its supporting index.
