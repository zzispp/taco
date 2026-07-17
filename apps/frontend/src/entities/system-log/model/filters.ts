import type { SystemLogLevel, SystemLogFilters, SystemLogFilterQuery } from './types';

import {
  parseLocalDateTimeRange,
  LOCAL_DATE_TIME_FILTER_ERROR,
} from 'src/shared/lib/local-date-time-filter';

const DEFAULT_LOOKBACK_MS = 24 * 60 * 60 * 1000;

export const SYSTEM_LOG_FILTER_ERROR = LOCAL_DATE_TIME_FILTER_ERROR;
export type SystemLogFilterError =
  (typeof SYSTEM_LOG_FILTER_ERROR)[keyof typeof SYSTEM_LOG_FILTER_ERROR];

export function createDefaultSystemLogFilters(now = new Date()): SystemLogFilters {
  return {
    keyword: '',
    levels: [],
    target: '',
    begin_time: toLocalDateTimeInput(new Date(now.getTime() - DEFAULT_LOOKBACK_MS)),
    end_time: toLocalDateTimeInput(now),
  };
}

export function toSystemLogQuery(
  draft: SystemLogFilters
):
  | Readonly<{ ok: true; query: SystemLogFilterQuery }>
  | Readonly<{ ok: false; error: SystemLogFilterError }> {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  const keyword = draft.keyword.trim();
  const target = draft.target.trim();
  const levels = uniqueLevels(draft.levels);
  return {
    ok: true,
    query: {
      ...(keyword && { keyword }),
      ...(target && { target }),
      ...(levels.length > 0 && { levels: levels.join(',') }),
      ...(dates.begin && { begin_time: dates.begin.toISOString() }),
      ...(dates.end && { end_time: dates.end.toISOString() }),
    },
  };
}

export function hasRequiredSystemLogRange(query: SystemLogFilterQuery) {
  return Boolean(query.begin_time && query.end_time);
}

function uniqueLevels(levels: readonly SystemLogLevel[]) {
  return [...new Set(levels)].sort();
}

function toLocalDateTimeInput(value: Date) {
  return `${value.getFullYear()}-${pad(value.getMonth() + 1)}-${pad(value.getDate())}T${pad(value.getHours())}:${pad(value.getMinutes())}:${pad(value.getSeconds())}.${pad(value.getMilliseconds(), 3)}`;
}

function pad(value: number, length = 2) {
  return value.toString().padStart(length, '0');
}
