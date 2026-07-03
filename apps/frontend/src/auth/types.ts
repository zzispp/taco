export type UserType = Record<string, any> | null;

export type AuthState = {
  user: UserType;
  loading: boolean;
  error: Error | null;
};

export type AuthContextValue = {
  user: UserType;
  loading: boolean;
  error: Error | null;
  authenticated: boolean;
  unauthenticated: boolean;
  checkUserSession?: () => Promise<void>;
};
