'use client';

import { useSetState } from 'minimal-shared/hooks';
import { useMemo, useEffect, useCallback } from 'react';

import { resolveServerAssetUrl } from 'src/shared/lib/asset-url';
import axios, { isAuthSessionRejected } from 'src/shared/api/http-client';

import {
  setSession,
  AuthContext,
  isValidToken,
  type AuthState,
  JWT_STORAGE_KEY,
  type SessionUser,
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

type AuthStateSetter = (value: Partial<AuthState>) => void;

export function AuthProvider({ children }: Props) {
  const { state, setState } = useSetState<AuthState>({
    user: null,
    error: null,
    loading: true,
  });

  const checkUserSession = useCheckUserSession(setState);

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
      user: state.user,
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

function useCheckUserSession(setState: AuthStateSetter) {
  return useCallback(async () => {
    await syncUserSession(setState);
  }, [setState]);
}

async function syncUserSession(setState: AuthStateSetter) {
  const session = await resolveSession();
  if (!session) {
    await setSession(null);
    setUnauthenticated(setState);
    return;
  }
  try {
    setState({
      user: await fetchSessionUser(session.access_token),
      error: null,
      loading: false,
    });
  } catch (error) {
    if (isAuthSessionRejected(error)) {
      await setSession(null);
      setUnauthenticated(setState);
      return;
    }
    throw error;
  }
}

async function fetchSessionUser(accessToken: string): Promise<SessionUser> {
  const res = await axios.get(AUTH_ME_ENDPOINT);
  const { user } = res.data as MeResponse;
  return {
    ...user,
    access_token: accessToken,
    displayName: user.nick_name || user.username,
    photoURL: resolveServerAssetUrl(user.avatar),
  };
}

function setUnauthenticated(setState: AuthStateSetter) {
  setState({ user: null, error: null, loading: false });
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

  try {
    const res = await axios.post(AUTH_REFRESH_ENDPOINT, { refresh_token });
    const session = res.data as TokenPairResponse;

    await setSession(session);

    return session;
  } catch (error) {
    if (isAuthSessionRejected(error)) {
      await setSession(null);
      return null;
    }
    throw error;
  }
}
