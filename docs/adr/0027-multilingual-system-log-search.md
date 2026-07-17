# ADR 0027: Multilingual System Log Search

- Status: Accepted
- Date: 2026-07-16

## Context

System-log messages and field values can contain English words, Chinese text, identifiers, and arbitrary error-code fragments. PostgreSQL standard full-text search is not sufficient for every matching mode.

## Decision

Keep the full-text-search index and add the PostgreSQL `pg_trgm` extension with a trigram GIN index for system-log search content. Use full-text search for word-oriented queries and trigram search for Chinese text, identifiers, and substring matching.

## Consequences

- Keyword search remains effective across mixed-language diagnostics.
- The migration requires PostgreSQL extension support and maintains two complementary search indexes.
- Query construction must choose the indexed search strategy rather than falling back to sequential scanning.
