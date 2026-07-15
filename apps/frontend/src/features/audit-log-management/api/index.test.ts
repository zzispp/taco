import { mutate } from 'swr';
import { it, vi, expect, describe, beforeEach } from 'vitest';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';

import { auditLogEndpoints } from 'src/entities/audit-log';

import {
  deleteLoginLog,
  cleanLoginLogs,
  deleteLoginLogs,
  exportLoginLogs,
  deleteOperationLog,
  cleanOperationLogs,
  unlockLoginAccount,
  deleteOperationLogs,
  exportOperationLogs,
} from './index';

vi.mock('src/shared/api/http-client', () => ({
  default: { delete: vi.fn(), post: vi.fn(), put: vi.fn() },
}));
vi.mock('src/shared/api/download', () => ({ downloadBlobResponse: vi.fn() }));
vi.mock('swr', () => ({ mutate: vi.fn() }));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(axios.delete).mockResolvedValue({ data: null } as never);
  vi.mocked(axios.put).mockResolvedValue({ data: null } as never);
  vi.mocked(axios.post).mockResolvedValue({ data: new Blob(), headers: {} } as never);
  vi.mocked(mutate).mockResolvedValue(undefined);
});

describe('audit cache refresh matrix', () => {
  it.each([
    ['delete operation', () => deleteOperationLog('operation-id'), ['operation']],
    ['delete operations', () => deleteOperationLogs(['operation-id']), ['operation']],
    ['clean operations', cleanOperationLogs, ['operation']],
    ['export operations', () => exportOperationLogs({}), ['operation']],
    ['delete login', () => deleteLoginLog('login-id'), ['login', 'operation']],
    ['delete logins', () => deleteLoginLogs(['login-id']), ['login', 'operation']],
    ['clean logins', cleanLoginLogs, ['login', 'operation']],
    ['export logins', () => exportLoginLogs({}), ['operation']],
    ['unlock account', () => unlockLoginAccount('admin'), ['operation']],
  ] as const)('refreshes the required lists after %s', async (_, action, expected) => {
    await action();

    expect(refreshedLists()).toEqual(expected);
  });
});

describe('audit log export', () => {
  it('uses the exact current validated operation-log query', async () => {
    const response = { data: new Blob(), headers: {} };
    vi.mocked(axios.post).mockResolvedValue(response as never);
    const query = {
      title: '用户管理',
      begin_time: '2026-07-13T00:00:00.000Z',
      end_time: '2026-07-13T10:00:00.000Z',
      sort_by: 'oper_time' as const,
      sort_order: 'desc' as const,
    };

    await exportOperationLogs(query);

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(
      auditLogEndpoints.operationLogsExport,
      null,
      { params: query, responseType: 'blob' }
    );
    expect(downloadBlobResponse).toHaveBeenCalledExactlyOnceWith(response, 'operation_logs.xlsx');
  });

  it('uses the exact current validated login-log query', async () => {
    const response = { data: new Blob(), headers: {} };
    vi.mocked(axios.post).mockResolvedValue(response as never);
    const query = { user_name: 'admin', status: 1 as const };

    await exportLoginLogs(query);

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(auditLogEndpoints.loginLogsExport, null, {
      params: query,
      responseType: 'blob',
    });
    expect(downloadBlobResponse).toHaveBeenCalledExactlyOnceWith(response, 'login_logs.xlsx');
  });
});

function refreshedLists() {
  const endpoints = [
    ['operation', auditLogEndpoints.operationLogs],
    ['login', auditLogEndpoints.loginLogs],
  ] as const;

  return vi.mocked(mutate).mock.calls.flatMap(([matcher]) => {
    if (typeof matcher !== 'function') throw new Error('Expected an SWR cache matcher');
    return endpoints.filter(([, endpoint]) => matcher([endpoint, {}])).map(([name]) => name);
  });
}

describe('login account unlock', () => {
  it('uses a side-effecting PUT endpoint', async () => {
    vi.mocked(axios.put).mockResolvedValue({ data: null } as never);

    await unlockLoginAccount('admin@example.com');

    expect(axios.put).toHaveBeenCalledExactlyOnceWith(
      auditLogEndpoints.loginLogUnlock('admin@example.com')
    );
  });
});
