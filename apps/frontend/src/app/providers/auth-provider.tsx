'use client';

import type { AuthState, SessionUser } from 'src/entities/session/model/types';

import { useSetState } from 'minimal-shared/hooks';
import { useMemo, useEffect, useCallback } from 'react';

import axios from 'src/shared/api/http-client';

import { AuthContext } from 'src/entities/session/model/auth-context';
import {
  setSession,
  isValidToken,
  JWT_STORAGE_KEY,
  JWT_REFRESH_STORAGE_KEY,
} from 'src/entities/session';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

const AUTH_ME_ENDPOINT = '/api/auth/me';
const AUTH_REFRESH_ENDPOINT = '/api/auth/refresh';

type TokenPairResponse = {
  access_token: string;
  refresh_token: string;
};

type MeResponse = {
  user: Omit<SessionUser, 'access_token' | 'displayName' | 'photoURL'>;
};

export function AuthProvider({ children }: Props) {
  const { state, setState } = useSetState<AuthState>({
    user: null,
    error: null,
    loading: true,
  });

  const checkUserSession = useCallback(async () => {
    const session = await resolveSession();

    if (!session) {
      await setSession(null);
      setState({ user: null, error: null, loading: false });
      return;
    }

    const res = await axios.get(AUTH_ME_ENDPOINT);
    const { user } = res.data as MeResponse;

    setState({
      user: {
        ...user,
        access_token: session.access_token,
        displayName: user.username,
      },
      error: null,
      loading: false,
    });
  }, [setState]);

  useEffect(() => {
    checkUserSession().catch((error: Error) => {
      console.error(error);
      setState({ error, loading: false });
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ----------------------------------------------------------------------

  const checkAuthenticated = state.user ? 'authenticated' : 'unauthenticated';

  const status = state.loading ? 'loading' : checkAuthenticated;

  const memoizedValue = useMemo(
    () => ({
      user: state.user ? { ...state.user, role: state.user?.role ?? 'admin' } : null,
      checkUserSession,
      error: state.error,
      loading: status === 'loading',
      authenticated: status === 'authenticated',
      unauthenticated: status === 'unauthenticated',
    }),
    [checkUserSession, state.error, state.user, status]
  );

  if (state.error) {
    throw state.error;
  }

  return <AuthContext value={memoizedValue}>{children}</AuthContext>;
}

async function resolveSession() {
  const access_token = localStorage.getItem(JWT_STORAGE_KEY);
  const refresh_token = localStorage.getItem(JWT_REFRESH_STORAGE_KEY);

  if (access_token && refresh_token && isValidToken(access_token)) {
    const session = { access_token, refresh_token };
    await setSession(session);
    return session;
  }

  if (!refresh_token || !isValidToken(refresh_token)) {
    return null;
  }

  const res = await axios.post(AUTH_REFRESH_ENDPOINT, { refresh_token });
  const session = res.data as TokenPairResponse;

  await setSession(session);

  return session;
}
