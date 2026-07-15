import { vi, it, expect, describe, beforeEach } from 'vitest';

const { headers, markAuthSessionEstablished, post } = vi.hoisted(() => ({
  post: vi.fn(),
  markAuthSessionEstablished: vi.fn(),
  headers: { common: {} as Record<string, string> },
}));

vi.mock('src/shared/api/http-client', () => ({
  default: {
    post,
    defaults: { headers },
  },
  isAuthSessionRejected: (error: unknown) =>
    typeof error === 'object' && error !== null && 'status' in error && error.status === 401,
  markAuthSessionEstablished,
  skipAuthSessionRecovery: () => ({ skipAuthSessionRecovery: true }),
}));

import { setSession, restoreSession, refreshSession } from './token';

const ACCESS_TOKEN = jwt({ exp: 4_102_444_800 });

describe('in-memory auth session', () => {
  beforeEach(() => {
    post.mockReset();
    markAuthSessionEstablished.mockReset();
    delete headers.common.Authorization;
  });

  it('keeps the access token in memory and removes legacy persisted tokens', async () => {
    const localStorage = storageSpy();

    await setSession({ access_token: ACCESS_TOKEN });

    expect(headers.common.Authorization).toBe(`Bearer ${ACCESS_TOKEN}`);
    expect(markAuthSessionEstablished).toHaveBeenCalledTimes(1);
    expect(localStorage.getItem).not.toHaveBeenCalled();
    expect(localStorage.setItem).not.toHaveBeenCalled();
    expect(localStorage.removeItem).toHaveBeenNthCalledWith(1, 'jwt_access_token');
    expect(localStorage.removeItem).toHaveBeenNthCalledWith(2, 'jwt_refresh_token');
  });

  it('restores a session through the HttpOnly refresh cookie', async () => {
    post.mockResolvedValue({ data: { access_token: ACCESS_TOKEN } });

    const session = await restoreSession();

    expect(post).toHaveBeenCalledWith('/api/auth/refresh', undefined, {
      skipAuthSessionRecovery: true,
    });
    expect(session).toEqual({ access_token: ACCESS_TOKEN });
    expect(headers.common.Authorization).toBe(`Bearer ${ACCESS_TOKEN}`);
  });

  it('returns null when cookie refresh is rejected', async () => {
    post.mockRejectedValue(Object.assign(new Error('unauthorized'), { status: 401 }));

    await expect(refreshSession()).resolves.toBeNull();
    expect(headers.common.Authorization).toBeUndefined();
  });

  it('exposes refresh infrastructure failures', async () => {
    const failure = new Error('upstream unavailable');
    post.mockRejectedValue(failure);

    await expect(refreshSession()).rejects.toBe(failure);
  });

  it('clears the in-memory authorization header', async () => {
    headers.common.Authorization = `Bearer ${ACCESS_TOKEN}`;

    await setSession(null);

    expect(headers.common.Authorization).toBeUndefined();
  });

  it('rejects an expired access token', async () => {
    const expired = jwt({ exp: 1 });
    headers.common.Authorization = `Bearer ${ACCESS_TOKEN}`;

    await expect(setSession({ access_token: expired })).rejects.toThrow('Invalid access token');
    expect(headers.common.Authorization).toBeUndefined();
  });
});

function storageSpy() {
  const storage = {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn(),
  };
  vi.stubGlobal('window', { localStorage: storage });
  vi.stubGlobal('localStorage', storage);
  return storage;
}

function jwt(payload: Record<string, unknown>) {
  const encoded = Buffer.from(JSON.stringify(payload)).toString('base64url');
  return `header.${encoded}.signature`;
}
