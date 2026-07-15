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

export type OnlineSessionFilters = {
  ipaddr: string;
  userName: string;
  loginLocation: string;
  browser: string;
  os: string;
  begin_time: string;
  end_time: string;
};

export type OnlineSessionQuery = Readonly<Partial<OnlineSessionFilters>>;
