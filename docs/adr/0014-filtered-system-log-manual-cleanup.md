# ADR 0014: Filtered System Log Manual Cleanup

- Status: Accepted
- Date: 2026-07-16

## Context

System logs are continuously written and retain operational evidence. An unconditional table-wide cleanup can destroy records needed for current investigation.

## Decision

Manual cleanup deletes only the records matched by the current system-log filters. A time range is required. The confirmation dialog shows the matching record count before deletion.

## Consequences

- Manual cleanup uses a filter-aware deletion API rather than a table-wide `DELETE`.
- The UI must prevent submission until the filter includes a valid time range.
- Operators can safely clear a defined incident window without deleting unrelated logs.
