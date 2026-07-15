import type { RoleUser } from 'src/entities/role';

import { mutate } from 'swr';
import { vi, it, expect, describe, beforeEach } from 'vitest';

import axios from 'src/shared/api/http-client';

import { roleEndpoints } from 'src/entities/role';

import { assignRoleUsers } from './index';

vi.mock('swr', () => ({ mutate: vi.fn() }));
vi.mock('src/shared/api/http-client', () => ({
  default: { get: vi.fn(), put: vi.fn() },
}));

beforeEach(() => vi.clearAllMocks());

describe('assignRoleUsers', () => {
  it('reads every allocated cursor batch before merging explicit user ids', async () => {
    const firstBatch = roleUsers(1, 100);
    const secondBatch = roleUsers(101, 1);
    vi.mocked(axios.get)
      .mockResolvedValueOnce(cursorResponse(firstBatch, 'next-100'))
      .mockResolvedValueOnce(cursorResponse(secondBatch, null));
    vi.mocked(axios.put).mockResolvedValue({} as never);

    await assignRoleUsers('role-1', ['user-50', 'user-102']);

    expect(axios.get).toHaveBeenNthCalledWith(1, roleEndpoints.users('role-1'), {
      params: { limit: 100, allocated: true },
    });
    expect(axios.get).toHaveBeenNthCalledWith(2, roleEndpoints.users('role-1'), {
      params: { limit: 100, cursor: 'next-100', allocated: true },
    });
    expect(axios.put).toHaveBeenCalledExactlyOnceWith(roleEndpoints.users('role-1'), {
      user_ids: [...firstBatch, ...secondBatch, user('user-102')].map((item) => item.user_id),
    });
    expect(mutate).toHaveBeenCalledOnce();
  });

  it('rejects an incomplete cursor response before updating role users', async () => {
    vi.mocked(axios.get).mockResolvedValueOnce({
      data: {
        items: [user('user-1')],
        next_cursor: null,
        previous_cursor: null,
        has_next: true,
        has_previous: false,
      },
    });

    await expect(assignRoleUsers('role-1', ['user-2'])).rejects.toThrow(
      'Role-user cursor response is inconsistent'
    );
    expect(axios.put).not.toHaveBeenCalled();
    expect(mutate).not.toHaveBeenCalled();
  });
});

function roleUsers(start: number, count: number): RoleUser[] {
  return Array.from({ length: count }, (_, index) => user(`user-${start + index}`));
}

function user(userId: string): RoleUser {
  return { user_id: userId } as RoleUser;
}

function cursorResponse(items: RoleUser[], nextCursor: string | null) {
  return {
    data: {
      items,
      next_cursor: nextCursor,
      previous_cursor: null,
      has_next: nextCursor !== null,
      has_previous: false,
    },
  };
}
