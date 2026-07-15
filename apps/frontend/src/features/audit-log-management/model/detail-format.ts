export type AuditDetailSource = Readonly<{
  oper_param: string | null;
  json_result: string | null;
  error_msg: string | null;
}>;

export function formatAuditDetailValue(value: string | null) {
  if (!value) return '';
  const trimmed = value.trim();
  if (!trimmed) return '';
  try {
    return JSON.stringify(JSON.parse(trimmed), null, 2);
  } catch {
    return value;
  }
}

export function auditDetailSections(detail: AuditDetailSource) {
  return [
    { key: 'request', value: formatAuditDetailValue(detail.oper_param) },
    { key: 'response', value: formatAuditDetailValue(detail.json_result) },
    { key: 'error', value: formatAuditDetailValue(detail.error_msg) },
  ] as const;
}
