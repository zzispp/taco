import type {
  LoginLogFilters,
  LoginLogFilterQuery,
  OperationLogFilters,
  OperationLogFilterQuery,
} from './types';

import {
  parseLocalDateTimeRange,
  LOCAL_DATE_TIME_FILTER_ERROR,
} from 'src/shared/lib/local-date-time-filter';

export const AUDIT_FILTER_ERROR = LOCAL_DATE_TIME_FILTER_ERROR;
export type AuditFilterError = (typeof AUDIT_FILTER_ERROR)[keyof typeof AUDIT_FILTER_ERROR];

export const DEFAULT_OPERATION_LOG_FILTERS: OperationLogFilters = Object.freeze({
  title: '',
  oper_name: '',
  oper_ip: '',
  business_type: '',
  status: '',
  begin_time: '',
  end_time: '',
});

export const DEFAULT_LOGIN_LOG_FILTERS: LoginLogFilters = Object.freeze({
  ipaddr: '',
  user_name: '',
  status: '',
  event_type: '',
  begin_time: '',
  end_time: '',
});

export const DEFAULT_OPERATION_LOG_QUERY: OperationLogFilterQuery = Object.freeze({});
export const DEFAULT_LOGIN_LOG_QUERY: LoginLogFilterQuery = Object.freeze({});

type FilterResult<T> =
  Readonly<{ ok: true; query: T }> | Readonly<{ ok: false; error: AuditFilterError }>;

export function toOperationLogQuery(
  draft: OperationLogFilters
): FilterResult<OperationLogFilterQuery> {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  const title = draft.title.trim();
  const operName = draft.oper_name.trim();
  const operIp = draft.oper_ip.trim();
  return {
    ok: true,
    query: {
      ...(title && { title }),
      ...(operName && { oper_name: operName }),
      ...(operIp && { oper_ip: operIp }),
      ...(draft.business_type !== '' && { business_type: draft.business_type }),
      ...(draft.status !== '' && { status: draft.status }),
      ...dateQuery(dates.begin, dates.end),
    },
  };
}

export function toLoginLogQuery(draft: LoginLogFilters): FilterResult<LoginLogFilterQuery> {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  const ipaddr = draft.ipaddr.trim();
  const userName = draft.user_name.trim();
  return {
    ok: true,
    query: {
      ...(ipaddr && { ipaddr }),
      ...(userName && { user_name: userName }),
      ...(draft.status !== '' && { status: draft.status }),
      ...(draft.event_type !== '' && { event_type: draft.event_type }),
      ...dateQuery(dates.begin, dates.end),
    },
  };
}

export function updateOperationLogFilterState(
  currentQuery: OperationLogFilterQuery,
  nextDraft: OperationLogFilters
) {
  return filterTransition(currentQuery, nextDraft, toOperationLogQuery);
}

export function updateLoginLogFilterState(
  currentQuery: LoginLogFilterQuery,
  nextDraft: LoginLogFilters
) {
  return filterTransition(currentQuery, nextDraft, toLoginLogQuery);
}

function filterTransition<Draft, Query>(
  currentQuery: Query,
  nextDraft: Draft,
  convert: (draft: Draft) => FilterResult<Query>
) {
  const result = convert(nextDraft);
  return {
    draft: nextDraft,
    query: result.ok ? result.query : currentQuery,
    error: result.ok ? null : result.error,
    resetTable: result.ok,
  } as const;
}

function dateQuery(begin: Date | null, end: Date | null) {
  return {
    ...(begin && { begin_time: begin.toISOString() }),
    ...(end && { end_time: end.toISOString() }),
  };
}
