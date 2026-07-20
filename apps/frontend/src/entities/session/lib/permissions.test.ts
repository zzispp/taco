import type { SessionUser } from '../model/types';

import { it, expect, describe } from 'vitest';

import { hasSessionPermission } from './permissions';

const REQUESTED_PERMISSION = 'system:operlog:list';

describe('session permission checking', () => {
  it('rejects missing sessions and permissions', () => {
    expect(hasSessionPermission(null, REQUESTED_PERMISSION)).toBe(false);
    expect(hasSessionPermission(sessionUser(), REQUESTED_PERMISSION)).toBe(false);
  });

  it('rejects the legacy wildcard permission for business users', () => {
    const wildcardOnly = sessionUser({ permissions: ['*:*:*'] });
    const explicitPermission = sessionUser({ permissions: ['*:*:*', REQUESTED_PERMISSION] });

    expect(hasSessionPermission(wildcardOnly, REQUESTED_PERMISSION)).toBe(false);
    expect(hasSessionPermission(explicitPermission, REQUESTED_PERMISSION)).toBe(true);
    expect(hasSessionPermission(explicitPermission, '*:*:*')).toBe(false);
  });

  it.each([
    ['installation owner marker', sessionUser({ is_installation_owner: true })],
    ['explicit permission', sessionUser({ permissions: [REQUESTED_PERMISSION] })],
  ])('allows a %s', (_, user) => {
    expect(hasSessionPermission(user, REQUESTED_PERMISSION)).toBe(true);
  });
});

function sessionUser(overrides: Partial<SessionUser> = {}): SessionUser {
  return {
    user_id: 'user-id',
    username: 'user',
    nick_name: 'User',
    dept_id: null,
    email: 'user@example.com',
    phonenumber: null,
    sex: '0',
    avatar: null,
    status: '0',
    is_active: true,
    is_installation_owner: false,
    auth_source: 'local',
    email_verified: true,
    remark: null,
    roles: [],
    role_ids: [],
    post_ids: [],
    permissions: [],
    access_token: 'token',
    displayName: 'User',
    ...overrides,
  };
}
