import { it, expect, describe } from 'vitest';

import {
  isAuditSortField,
  LOGIN_LOG_SORT_FIELDS,
  createLoginLogListQuery,
  OPERATION_LOG_SORT_FIELDS,
  createOperationLogListQuery,
} from './sorting';

describe('audit log server sorting', () => {
  it('adds only an operation-log whitelist sort to the validated filters', () => {
    expect(
      createOperationLogListQuery(
        { oper_name: 'admin' },
        { sort_by: 'oper_time', sort_order: 'desc' }
      )
    ).toEqual({ oper_name: 'admin', sort_by: 'oper_time', sort_order: 'desc' });
    expect(isAuditSortField('method', OPERATION_LOG_SORT_FIELDS)).toBe(false);
    expect(isAuditSortField('title', OPERATION_LOG_SORT_FIELDS)).toBe(false);
  });

  it('supports the login-log whitelist without mutating filters', () => {
    const filters = { status: 1 as const };
    const query = createLoginLogListQuery(filters, {
      sort_by: 'login_time',
      sort_order: 'asc',
    });

    expect(query).toEqual({ status: 1, sort_by: 'login_time', sort_order: 'asc' });
    expect(filters).toEqual({ status: 1 });
    expect(isAuditSortField('oper_time', LOGIN_LOG_SORT_FIELDS)).toBe(false);
  });
});
