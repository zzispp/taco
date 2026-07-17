# ADR 0018: Centralized Sensitive Data Redaction

- Status: Accepted
- Date: 2026-07-16

## Context

Audit request snapshots already redact sensitive field names, while tracing has no shared redaction layer. System logs retain and export structured event fields, so duplicate or inconsistent policies would expose data.

## Decision

Add a dependency-free `kernel::redaction` module that owns sensitive-key normalization, sensitive-key detection, the redaction marker, and recursive JSON field redaction. The audit HTTP sanitizer keeps request-body, URL, and truncation logic but delegates generic field redaction to `kernel`. Tracing applies the same policy before events reach any tracing sink, including standard output and asynchronous persistence (ADR 0037).

## Consequences

- Sensitive values do not enter standard output, the system-log queue, database, detail view, search index, or export.
- Audit and tracing use one sensitive-key policy without coupling tracing to HTTP or audit domains.
- No new shared crate is introduced for a small, dependency-free cross-cutting primitive.
