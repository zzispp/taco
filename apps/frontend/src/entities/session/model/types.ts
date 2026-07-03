export type SessionUser = {
  id: string;
  username: string;
  email: string;
  role: string;
  is_active: boolean;
  auth_source: string;
  email_verified: boolean;
  system: boolean;
  access_token: string;
  displayName: string;
  photoURL?: string;
};

export type AuthState = {
  user: SessionUser | null;
  loading: boolean;
  error: Error | null;
};

export type AuthContextValue = {
  user: SessionUser | null;
  loading: boolean;
  error: Error | null;
  authenticated: boolean;
  unauthenticated: boolean;
  checkUserSession?: () => Promise<void>;
};
