import axios, {
  isAuthSessionRejected,
  skipAuthSessionRecovery,
  markAuthSessionEstablished,
} from 'src/shared/api/http-client';

// ----------------------------------------------------------------------

const AUTH_REFRESH_ENDPOINT = '/api/auth/refresh';
const LEGACY_ACCESS_TOKEN_KEY = 'jwt_access_token';
const LEGACY_REFRESH_TOKEN_KEY = 'jwt_refresh_token';

export type JwtSession = {
  access_token: string;
};

export function jwtDecode(token: string) {
  try {
    if (!token) return null;

    const parts = token.split('.');
    if (parts.length < 2) {
      throw new Error('Invalid token!');
    }

    const base64Url = parts[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const decoded = JSON.parse(atob(base64));

    return decoded;
  } catch (error) {
    console.error('Error decoding token:', error);
    throw error;
  }
}

// ----------------------------------------------------------------------

export function isValidToken(access_token: string) {
  if (!access_token) {
    return false;
  }

  try {
    const decoded = jwtDecode(access_token);

    if (!decoded || !('exp' in decoded)) {
      return false;
    }

    const currentTime = Date.now() / 1000;

    return decoded.exp > currentTime;
  } catch (error) {
    console.error('Error during token validation:', error);
    return false;
  }
}

// ----------------------------------------------------------------------

export async function restoreSession(): Promise<JwtSession | null> {
  clearLegacyTokens();
  return refreshSession();
}

export async function refreshSession(): Promise<JwtSession | null> {
  try {
    const response = await axios.post<JwtSession>(
      AUTH_REFRESH_ENDPOINT,
      undefined,
      skipAuthSessionRecovery()
    );
    await setSession(response.data);
    return response.data;
  } catch (error) {
    if (!isAuthSessionRejected(error)) throw error;
    await setSession(null);
    return null;
  }
}

export async function setSession(session: JwtSession | null) {
  clearLegacyTokens();
  if (!session) {
    delete axios.defaults.headers.common.Authorization;
    return;
  }

  assertSession(session);
  if (!isValidToken(session.access_token)) {
    delete axios.defaults.headers.common.Authorization;
    throw new Error('Invalid access token!');
  }

  axios.defaults.headers.common.Authorization = `Bearer ${session.access_token}`;
  markAuthSessionEstablished();
}

function clearLegacyTokens() {
  if (typeof window === 'undefined') return;
  window.localStorage.removeItem(LEGACY_ACCESS_TOKEN_KEY);
  window.localStorage.removeItem(LEGACY_REFRESH_TOKEN_KEY);
}

function assertSession(session: JwtSession) {
  if (!session.access_token) {
    throw new Error('Access token not found in response');
  }
}
