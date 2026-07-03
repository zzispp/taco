export const userEndpoints = {
  users: '/api/users',
  user: (id: string) => `/api/users/${id}`,
} as const;
