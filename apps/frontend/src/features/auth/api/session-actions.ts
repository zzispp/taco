'use client';

import axios from 'src/shared/api/http-client';

import { setSession, trimCredential } from 'src/entities/session';

const AUTH_SIGN_IN_ENDPOINT = '/api/auth/sign-in';
const AUTH_SIGN_UP_ENDPOINT = '/api/auth/sign-up';
const AUTH_LOGOUT_ENDPOINT = '/api/auth/logout';

// ----------------------------------------------------------------------

export type SignInParams = {
  identifier: string;
  password: string;
  captchaToken?: string;
};

export type SignUpParams = {
  username: string;
  email: string;
  password: string;
  captchaToken?: string;
};

type AuthSessionResponse = {
  access_token: string;
  refresh_token: string;
};

/** **************************************
 * Sign in
 *************************************** */
export const signInWithPassword = async ({
  identifier,
  password,
  captchaToken,
}: SignInParams): Promise<void> => {
  try {
    const params = {
      identifier: trimCredential(identifier),
      password: trimCredential(password),
      ...(captchaToken ? { captcha_token: captchaToken.trim() } : {}),
    };

    const res = await axios.post(AUTH_SIGN_IN_ENDPOINT, params);

    await setSession(requireAuthSession(res.data));
  } catch (error) {
    console.error('Error during sign in:', error);
    throw error;
  }
};

/** **************************************
 * Sign up
 *************************************** */
export const signUp = async ({
  username,
  email,
  password,
  captchaToken,
}: SignUpParams): Promise<void> => {
  const params = {
    username: trimCredential(username),
    email: trimCredential(email),
    password: trimCredential(password),
    ...(captchaToken ? { captcha_token: captchaToken.trim() } : {}),
  };

  try {
    const res = await axios.post(AUTH_SIGN_UP_ENDPOINT, params);

    await setSession(requireAuthSession(res.data));
  } catch (error) {
    console.error('Error during sign up:', error);
    throw error;
  }
};

/** **************************************
 * Sign out
 *************************************** */
export const signOut = async (): Promise<void> => {
  try {
    await axios.post(AUTH_LOGOUT_ENDPOINT);
    await setSession(null);
  } catch (error) {
    console.error('Error during sign out:', error);
    throw error;
  }
};

function requireAuthSession(payload: AuthSessionResponse): AuthSessionResponse {
  if (!payload.access_token || !payload.refresh_token) {
    throw new Error('Auth tokens not found in response');
  }

  return payload;
}
