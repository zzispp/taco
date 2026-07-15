import type { RoleSummary } from 'src/entities/role';

export type SessionUser = {
  user_id: string;
  username: string;
  nick_name: string;
  dept_id: string | null;
  email: string;
  phonenumber: string | null;
  sex: string;
  avatar: string | null;
  status: string;
  is_active: boolean;
  auth_source: string;
  email_verified: boolean;
  remark: string | null;
  roles: RoleSummary[];
  role_ids: string[];
  post_ids: string[];
  permissions: string[];
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
  checkUserSession: () => Promise<void>;
};
