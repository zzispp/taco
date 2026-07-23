# Audit file mutations and downloads without read noise

File Management writes existing operation-audit records for business mutations, quota changes, and downloads, but not for lists, metadata reads, thumbnails, Inline Preview, Upload Session initialization, or individual Upload Parts. Only upload completion and cancellation are session-level upload audit events. This preserves traceability for data changes and export while avoiding an audit stream dominated by retryable initialization, rendering, and resumable-transfer traffic.
