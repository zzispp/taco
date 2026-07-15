'use client';

import { useSetState } from 'minimal-shared/hooks';
import { useMemo, useEffect, useCallback } from 'react';

import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';
import { resolveServerAssetUrl } from 'src/shared/lib/asset-url';
import axios, {
  isAuthSessionRejected,
  registerAuthSessionRecovery,
} from 'src/shared/api/http-client';

import {
  setSession,
  AuthContext,
  type AuthState,
  restoreSession,
  refreshSession,
  type SessionUser,
} from 'src/entities/session';

import { endSessionAfterTerminalRejection } from './auth-session-lifecycle';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

const AUTH_ME_ENDPOINT = '/api/auth/me';

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

  const checkUserSession = useAuthSessionLifecycle(setState);

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

function useAuthSessionLifecycle(setState: AuthStateSetter) {
  const router = useRouter();
  const checkUserSession = useCheckUserSession(setState);
  const refreshAccessToken = useCallback(
    async () => (await refreshSession())?.access_token ?? null,
    []
  );
  const endSession = useCallback(
    () =>
      endSessionAfterTerminalRejection({
        clearSession: () => setSession(null),
        refreshAuthState: checkUserSession,
        redirectToSignIn: () => router.replace(paths.auth.jwt.signIn),
      }),
    [checkUserSession, router]
  );

  useEffect(
    () =>
      registerAuthSessionRecovery({
        refreshAccessToken,
        onTerminalSessionRejected: endSession,
      }),
    [endSession, refreshAccessToken]
  );

  useEffect(() => {
    checkUserSession().catch((error: Error) => {
      console.error(error);
      setState({ error, loading: false });
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return checkUserSession;
}

function useCheckUserSession(setState: AuthStateSetter) {
  return useCallback(async () => {
    await syncUserSession(setState);
  }, [setState]);
}

async function syncUserSession(setState: AuthStateSetter) {
  const session = await restoreSession();
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
