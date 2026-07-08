import type { OnlineSessionFilters } from 'src/entities/online-session';

export const DEFAULT_FILTERS: OnlineSessionFilters = {
  ipaddr: '',
  userName: '',
  loginLocation: '',
  browser: '',
  os: '',
  begin_time: '',
  end_time: '',
};
