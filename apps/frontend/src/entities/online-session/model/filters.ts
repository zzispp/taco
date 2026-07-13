import type { OnlineSessionQuery, OnlineSessionFilters } from './types';

import {
  parseLocalDateTimeRange,
  LOCAL_DATE_TIME_FILTER_ERROR,
} from 'src/shared/lib/local-date-time-filter';

export const ONLINE_SESSION_FILTER_ERROR = LOCAL_DATE_TIME_FILTER_ERROR;

export type OnlineSessionFilterError =
  (typeof ONLINE_SESSION_FILTER_ERROR)[keyof typeof ONLINE_SESSION_FILTER_ERROR];

export const DEFAULT_ONLINE_SESSION_FILTERS: OnlineSessionFilters = Object.freeze({
  ipaddr: '',
  userName: '',
  loginLocation: '',
  browser: '',
  os: '',
  begin_time: '',
  end_time: '',
});

export const DEFAULT_ONLINE_SESSION_QUERY: OnlineSessionQuery = Object.freeze({});

export type OnlineSessionFilterResult =
  | Readonly<{ ok: true; query: OnlineSessionQuery }>
  | Readonly<{ ok: false; error: OnlineSessionFilterError }>;

export function toOnlineSessionQuery(draft: OnlineSessionFilters): OnlineSessionFilterResult {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  return {
    ok: true,
    query: {
      ...trimmedTextQuery(draft),
      ...(dates.begin && { begin_time: dates.begin.toISOString() }),
      ...(dates.end && { end_time: dates.end.toISOString() }),
    },
  };
}

export function updateOnlineSessionFilterState(
  currentQuery: OnlineSessionQuery,
  nextDraft: OnlineSessionFilters
) {
  const result = toOnlineSessionQuery(nextDraft);
  return {
    draft: nextDraft,
    query: result.ok ? result.query : currentQuery,
    error: result.ok ? null : result.error,
    resetTable: result.ok,
  } as const;
}

function trimmedTextQuery(draft: OnlineSessionFilters): OnlineSessionQuery {
  const entries = [
    ['ipaddr', draft.ipaddr],
    ['userName', draft.userName],
    ['loginLocation', draft.loginLocation],
    ['browser', draft.browser],
    ['os', draft.os],
  ] as const;
  return Object.fromEntries(
    entries.map(([key, value]) => [key, value.trim()]).filter(([, value]) => value)
  );
}
