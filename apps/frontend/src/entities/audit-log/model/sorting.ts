import type {
  AuditSortOrder,
  LoginLogListQuery,
  LoginLogSortField,
  LoginLogFilterQuery,
  OperationLogListQuery,
  OperationLogSortField,
  OperationLogFilterQuery,
} from './types';

export const OPERATION_LOG_SORT_FIELDS = [
  'business_type',
  'oper_name',
  'status',
  'oper_time',
  'cost_time',
] as const satisfies readonly OperationLogSortField[];

export const LOGIN_LOG_SORT_FIELDS = [
  'user_name',
  'ipaddr',
  'status',
  'login_time',
] as const satisfies readonly LoginLogSortField[];

export function isAuditSortField<T extends string>(
  value: string,
  fields: readonly T[]
): value is T {
  return fields.some((field) => field === value);
}

export function createOperationLogListQuery(
  filters: OperationLogFilterQuery,
  sort: Readonly<{ sort_by: OperationLogSortField; sort_order: AuditSortOrder }>
): OperationLogListQuery {
  return { ...filters, ...sort };
}

export function createLoginLogListQuery(
  filters: LoginLogFilterQuery,
  sort: Readonly<{ sort_by: LoginLogSortField; sort_order: AuditSortOrder }>
): LoginLogListQuery {
  return { ...filters, ...sort };
}
