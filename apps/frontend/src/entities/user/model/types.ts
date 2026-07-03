export type SystemUser = {
  id: string;
  username: string;
  email: string;
  role: string;
  is_active: boolean;
  auth_source: string;
  email_verified: boolean;
  system: boolean;
};

export type UserInput = {
  username: string;
  password: string;
  email: string;
  role: string;
  is_active: boolean;
};
