export type ApiPermission = {
  id: string;
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  group: string;
  enabled: boolean;
  system: boolean;
};

export type ApiPermissionInput = {
  code: string;
  method: string;
  path_pattern: string;
  name: string;
  group: string;
  enabled: boolean;
};
