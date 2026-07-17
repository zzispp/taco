# ADR 0030: No Local Log Migration

- Status: Accepted
- Date: 2026-07-16

## Context

Existing local tracing files are formatted text and do not carry the structured fields, target metadata, indexed search content, or centralized redaction guarantees of the new system-log model.

## Decision

Do not migrate local rolling log files. PostgreSQL system-log persistence begins when the new version is deployed.

## Consequences

- No compatibility parser or historical backfill path is introduced.
- Persisted logs have one reliable structure and redaction policy from their first record.
- Local file configuration and appender code can be removed completely.
