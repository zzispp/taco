import { vi, it, expect, describe } from 'vitest';

import { auditDetailSections, formatAuditDetailValue } from './detail-format';
import { resetAuditTableSort, resetAuditMutationCursor } from './table-actions';
import { AUDIT_PERMISSION, auditLogCapabilities, loginLogSelectionActions } from './permissions';

describe('audit log permissions and selection', () => {
  it('maps exact RuoYi permissions to operation-log capabilities', () => {
    const permissions = new Set<string>([
      AUDIT_PERMISSION.OPERATION_LIST,
      AUDIT_PERMISSION.OPERATION_QUERY,
      AUDIT_PERMISSION.OPERATION_EXPORT,
    ]);

    expect(auditLogCapabilities((permission) => permissions.has(permission))).toEqual({
      operation: { list: true, query: true, remove: false, export: true },
      login: { list: false, remove: false, export: false, unlock: false },
    });
  });

  it('maps all login-log permissions through the production capability checker', () => {
    const permissions = new Set<string>([
      AUDIT_PERMISSION.LOGIN_LIST,
      AUDIT_PERMISSION.LOGIN_REMOVE,
      AUDIT_PERMISSION.LOGIN_EXPORT,
      AUDIT_PERMISSION.LOGIN_UNLOCK,
    ]);

    expect(auditLogCapabilities((permission) => permissions.has(permission)).login).toEqual({
      list: true,
      remove: true,
      export: true,
      unlock: true,
    });
  });

  it('enables unlock only for one selected row and explicit unlock permission', () => {
    expect(loginLogSelectionActions([], true, true)).toEqual({
      canDelete: false,
      canUnlock: false,
    });
    expect(loginLogSelectionActions(['a'], true, true)).toEqual({
      canDelete: true,
      canUnlock: true,
    });
    expect(loginLogSelectionActions(['a', 'b'], true, true)).toEqual({
      canDelete: true,
      canUnlock: false,
    });
    expect(loginLogSelectionActions(['a'], true, false)).toEqual({
      canDelete: true,
      canUnlock: false,
    });
  });
});

describe('operation-log detail formatting', () => {
  it('pretty prints JSON and preserves non-JSON text', () => {
    expect(formatAuditDetailValue('{"password":"***","items":[1]}')).toBe(
      '{\n  "password": "***",\n  "items": [\n    1\n  ]\n}'
    );
    expect(formatAuditDetailValue('validation failed')).toBe('validation failed');
    expect(formatAuditDetailValue(null)).toBe('');
  });

  it('keeps request response and error as distinct sections', () => {
    expect(
      auditDetailSections({ oper_param: '{"id":1}', json_result: '{"ok":true}', error_msg: '' })
    ).toEqual([
      { key: 'request', value: '{\n  "id": 1\n}' },
      { key: 'response', value: '{\n  "ok": true\n}' },
      { key: 'error', value: '' },
    ]);
  });
});

describe('audit table reset behavior', () => {
  it.each([
    ['oper_time', 'operation logs'],
    ['login_time', 'login logs'],
  ])('restores descending default sort for %s', (field) => {
    const table = { setOrderBy: vi.fn(), setOrder: vi.fn() };

    resetAuditTableSort(table, field);

    expect(table.setOrderBy).toHaveBeenCalledExactlyOnceWith(field);
    expect(table.setOrder).toHaveBeenCalledExactlyOnceWith('desc');
  });

  it('resets after self-audited mutations without predicting list position', () => {
    const table = { onResetCursor: vi.fn(), setSelected: vi.fn() };

    resetAuditMutationCursor(table);

    expect(table.onResetCursor).toHaveBeenCalledExactlyOnceWith();
    expect(table.setSelected).toHaveBeenCalledExactlyOnceWith([]);
  });
});
