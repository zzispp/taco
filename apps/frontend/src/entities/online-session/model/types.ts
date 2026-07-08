export type OnlineSession = {
  tokenId: string;
  deptName: string | null;
  userName: string;
  ipaddr: string;
  loginLocation: string;
  browser: string;
  os: string;
  loginTime: number;
};

export type OnlineSessionsResponse = {
  rows: OnlineSession[];
  total: number;
};

export type OnlineSessionFilters = {
  ipaddr: string;
  userName: string;
};
