import axios from 'src/shared/api/http-client';

import { JWT_STORAGE_KEY, JWT_REFRESH_STORAGE_KEY } from './constants';

// ----------------------------------------------------------------------

export type JwtSession = {
  access_token: string;
  refresh_token: string;
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

export async function setSession(session: JwtSession | null) {
  try {
    if (session) {
      assertSession(session);
      localStorage.setItem(JWT_STORAGE_KEY, session.access_token);
      localStorage.setItem(JWT_REFRESH_STORAGE_KEY, session.refresh_token);

      axios.defaults.headers.common.Authorization = `Bearer ${session.access_token}`;

      const decodedToken = jwtDecode(session.access_token);

      if (!decodedToken || !('exp' in decodedToken)) {
        throw new Error('Invalid access token!');
      }
    } else {
      localStorage.removeItem(JWT_STORAGE_KEY);
      localStorage.removeItem(JWT_REFRESH_STORAGE_KEY);
      delete axios.defaults.headers.common.Authorization;
    }
  } catch (error) {
    console.error('Error during set session:', error);
    throw error;
  }
}

function assertSession(session: JwtSession) {
  if (!session.access_token || !session.refresh_token) {
    throw new Error('Auth tokens not found in response');
  }
}
