# ADR 0036: Lossless System Log XLSX Export

- Status: Accepted
- Date: 2026-07-17

## Context

The system-log event limit permits messages and structured fields larger than
an XLSX cell. A normal worksheet cell stores at most 32,767 characters, and a
worksheet stores at most 1,048,576 rows. Failing an entire export or silently
truncating evidence violates the time-bounded, complete-export contract.

## Decision

System-log XLSX exports are lossless. The primary worksheet retains the fixed
record columns and a reference for values that exceed an XLSX cell. Continuation
worksheets store each long message or fields JSON value as ordered chunks keyed
by `log_id`, value kind, and chunk sequence. Additional primary and continuation
worksheets are created before the Excel row limit is reached.

## Consequences

- Every accepted system-log record remains exportable without truncation.
- Consumers can reconstruct each long value deterministically from its ordered
  continuation chunks.
- Export tests must cover single-cell and worksheet-row limits.
