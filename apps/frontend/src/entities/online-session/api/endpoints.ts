export const onlineSessionEndpoints = {
  list: '/api/system/online/list',
  forceLogout: (tokenId: string) => `/api/system/online/${tokenId}`,
} as const;
