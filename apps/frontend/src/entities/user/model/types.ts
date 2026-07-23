import type { Post, TreeSelectNode } from 'src/entities/system';
import type { RoleOption, RoleSummary } from 'src/entities/role';

export type SystemUser = {
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
  create_time: string;
};

export type UserInput = {
  username: string;
  password?: string;
  nick_name: string;
  dept_id: string | null;
  email: string;
  phonenumber: string | null;
  sex: string;
  status: string;
  remark: string | null;
  role_ids: string[];
  post_ids: string[];
};

export type UserFormOptions = {
  roles: RoleOption[];
  posts: Post[];
  depts: TreeSelectNode[];
};

export type UserRolesPayload = {
  role_ids: string[];
};

export type UserImportResult = {
  success_count: number;
  message: string;
};

export type AccountProfile = {
  user: SystemUser;
  role_group: string;
  post_group: string;
  dept_name: string | null;
};

export type ProfileInput = {
  nick_name: string;
  phonenumber: string | null;
  email: string;
  sex: string;
};
