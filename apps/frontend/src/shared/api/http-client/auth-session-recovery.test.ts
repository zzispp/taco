import { it, expect, afterEach } from 'vitest';

import axios, { skipAuthSessionRecovery, registerAuthSessionRecovery } from '../http-client';
import {
  response,
  deferred,
  unauthorized,
  authorization,
  restoreAuthorization,
} from './auth-session-recovery.test-support';

const FRESH_TOKEN = 'fresh-token';
const STALE_AUTHORIZATION = 'Bearer stale-token';
const FRESH_AUTHORIZATION = `Bearer ${FRESH_TOKEN}`;
let unregisterRecovery: (() => void) | undefined;

const originalAdapter = axios.defaults.adapter;
const originalAuthorization = axios.defaults.headers.common.Authorization;

afterEach(() => {
  unregisterRecovery?.();
  unregisterRecovery = undefined;
  axios.defaults.adapter = originalAdapter;
  restoreAuthorization(originalAuthorization);
});

it('refreshes once, then retries a protected request with the new access token', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  const seenAuthorizations: Array<string | undefined> = [];
  let refreshCalls = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      return FRESH_TOKEN;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    seenAuthorizations.push(authorization(config));
    if (authorization(config) === STALE_AUTHORIZATION) throw unauthorized(config);
    return response(config, { recovered: true });
  };

  const result = await axios.get<{ recovered: boolean }>('/protected');

  expect(result.data).toEqual({ recovered: true });
  expect(refreshCalls).toBe(1);
  expect(seenAuthorizations).toEqual([STALE_AUTHORIZATION, FRESH_AUTHORIZATION]);
  expect(terminalRejections).toBe(0);
});

it('shares one in-flight refresh across concurrent protected request failures', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  const initialFailuresReached = deferred<void>();
  const releaseInitialFailures = deferred<void>();
  const refreshStarted = deferred<void>();
  const refreshResult = deferred<string>();
  let initialFailures = 0;
  let refreshCalls = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      refreshStarted.resolve();
      return refreshResult.promise;
    },
    onTerminalSessionRejected: async () => undefined,
  });
  axios.defaults.adapter = async (config) => {
    if (authorization(config) !== STALE_AUTHORIZATION) return response(config, { recovered: true });

    initialFailures += 1;
    if (initialFailures === CONCURRENT_REQUEST_COUNT) initialFailuresReached.resolve();
    await releaseInitialFailures.promise;
    throw unauthorized(config);
  };

  const requests = [axios.get('/protected/one'), axios.get('/protected/two')];
  await initialFailuresReached.promise;
  releaseInitialFailures.resolve();
  await refreshStarted.promise;

  expect(refreshCalls).toBe(1);

  refreshResult.resolve(FRESH_TOKEN);
  await expect(Promise.all(requests)).resolves.toHaveLength(CONCURRENT_REQUEST_COUNT);
  expect(refreshCalls).toBe(1);
});

it('handles a refresh failure as one terminal session rejection without recursive recovery', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  let protectedRequests = 0;
  let refreshRequests = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshRequests += 1;
      await axios.post('/api/auth/refresh', null, skipAuthSessionRecovery());
      return FRESH_TOKEN;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    if (config.url === '/api/auth/refresh') throw unauthorized(config);
    protectedRequests += 1;
    throw unauthorized(config);
  };

  await expect(axios.get('/protected')).rejects.toMatchObject({
    status: 401,
    code: 'unauthorized',
    details: '未登录或登录已失效',
  });

  expect(protectedRequests).toBe(1);
  expect(refreshRequests).toBe(1);
  expect(terminalRejections).toBe(1);
});

it('surfaces a shared refresh infrastructure failure without ending the session', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  const refreshStarted = deferred<void>();
  const refreshFailure = deferred<never>();
  const infrastructureFailure = new Error('refresh infrastructure failed');
  let refreshCalls = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      refreshStarted.resolve();
      return refreshFailure.promise;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    throw unauthorized(config);
  };

  const results = Promise.allSettled([axios.get('/protected/one'), axios.get('/protected/two')]);
  await refreshStarted.promise;
  refreshFailure.reject(infrastructureFailure);

  const settled = await results;
  expect(settled.map((result) => result.status)).toEqual(['rejected', 'rejected']);
  expect(settled.map((result) => (result as PromiseRejectedResult).reason)).toEqual([
    infrastructureFailure,
    infrastructureFailure,
  ]);
  expect(refreshCalls).toBe(1);
  expect(terminalRejections).toBe(0);
});

it('retries each original request at most once before terminal rejection', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  let protectedRequests = 0;
  let refreshCalls = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      return FRESH_TOKEN;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    protectedRequests += 1;
    throw unauthorized(config);
  };

  await expect(axios.get('/protected')).rejects.toMatchObject({
    status: 401,
    code: 'unauthorized',
  });

  expect(protectedRequests).toBe(2);
  expect(refreshCalls).toBe(1);
  expect(terminalRejections).toBe(1);
});

it('does not recover anonymous 401 responses', async () => {
  delete axios.defaults.headers.common.Authorization;
  let refreshCalls = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      return FRESH_TOKEN;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    throw unauthorized(config);
  };

  await expect(axios.get('/anonymous')).rejects.toMatchObject({
    status: 401,
    code: 'unauthorized',
  });

  expect(refreshCalls).toBe(0);
  expect(terminalRejections).toBe(0);
});

it('does not recover skipped authentication entry requests with a stale bearer header', async () => {
  axios.defaults.headers.common.Authorization = STALE_AUTHORIZATION;
  let refreshCalls = 0;
  let terminalRejections = 0;

  unregisterRecovery = registerAuthSessionRecovery({
    refreshAccessToken: async () => {
      refreshCalls += 1;
      return FRESH_TOKEN;
    },
    onTerminalSessionRejected: async () => {
      terminalRejections += 1;
    },
  });
  axios.defaults.adapter = async (config) => {
    throw unauthorized(config);
  };

  await expect(
    axios.post('/api/auth/sign-in', {}, skipAuthSessionRecovery())
  ).rejects.toMatchObject({
    status: 401,
    code: 'unauthorized',
  });

  expect(refreshCalls).toBe(0);
  expect(terminalRejections).toBe(0);
});
const CONCURRENT_REQUEST_COUNT = 2;
