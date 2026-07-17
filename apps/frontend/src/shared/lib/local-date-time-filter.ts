const LOCAL_DATE_TIME_PATTERN =
  /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2})(?:\.(\d{1,3}))?)?$/;
const MONTH_INDEX_OFFSET = 1;
const DATE_PART = {
  YEAR: 1,
  MONTH: 2,
  DAY: 3,
  HOUR: 4,
  MINUTE: 5,
  SECOND: 6,
  MILLISECOND: 7,
} as const;

export const INVALID_LOCAL_DATE_TIME_DRAFT = 'invalid';

export const LOCAL_DATE_TIME_FILTER_ERROR = {
  INVALID_DATE_TIME: 'invalid_date_time',
  INVALID_RANGE: 'invalid_range',
} as const;

export const LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY = {
  [LOCAL_DATE_TIME_FILTER_ERROR.INVALID_DATE_TIME]: 'dateTimeFilters.invalidDateTime',
  [LOCAL_DATE_TIME_FILTER_ERROR.INVALID_RANGE]: 'dateTimeFilters.invalidTimeRange',
} as const;

export type LocalDateTimeFilterError =
  (typeof LOCAL_DATE_TIME_FILTER_ERROR)[keyof typeof LOCAL_DATE_TIME_FILTER_ERROR];

export type LocalDateTimeRangeResult =
  | Readonly<{ ok: true; begin: Date | null; end: Date | null }>
  | Readonly<{ ok: false; error: LocalDateTimeFilterError }>;

export type LocalDateTimeFilterDraft = Readonly<
  Record<string, string> & {
    begin_time: string;
    end_time: string;
  }
>;

export type LocalDateTimeFilterQuery = Readonly<Record<string, string>>;

export type LocalDateTimeFilterState<T extends LocalDateTimeFilterDraft> = Readonly<{
  draft: T;
  query: LocalDateTimeFilterQuery;
  error: LocalDateTimeFilterError | null;
}>;

export type LocalDateTimeFilterTransition<T extends LocalDateTimeFilterDraft> = Readonly<{
  state: LocalDateTimeFilterState<T>;
  resetTable: boolean;
}>;

type LocalDateTimeFilterQueryResult =
  | Readonly<{ ok: true; query: LocalDateTimeFilterQuery }>
  | Readonly<{ ok: false; error: LocalDateTimeFilterError }>;

export function parseLocalDateTimeRange(
  beginValue: string,
  endValue: string
): LocalDateTimeRangeResult {
  const begin = beginValue ? parseLocalDateTime(beginValue) : null;
  const end = endValue ? parseLocalDateTime(endValue) : null;
  if ((beginValue && !begin) || (endValue && !end)) {
    return { ok: false, error: LOCAL_DATE_TIME_FILTER_ERROR.INVALID_DATE_TIME };
  }
  if (begin && end && begin.getTime() > end.getTime()) {
    return { ok: false, error: LOCAL_DATE_TIME_FILTER_ERROR.INVALID_RANGE };
  }
  return { ok: true, begin, end };
}

export function createLocalDateTimeFilterState<T extends LocalDateTimeFilterDraft>(
  draft: T
): LocalDateTimeFilterState<T> {
  const result = toUtcLocalDateTimeFilterQuery(draft);
  if (!result.ok) throw new Error(`Invalid initial local date-time filter: ${result.error}`);
  return { draft, query: result.query, error: null };
}

export function updateLocalDateTimeFilterState<T extends LocalDateTimeFilterDraft>(
  currentQuery: LocalDateTimeFilterQuery,
  nextDraft: T
) {
  const result = toUtcLocalDateTimeFilterQuery(nextDraft);
  return {
    draft: nextDraft,
    query: result.ok ? result.query : currentQuery,
    error: result.ok ? null : result.error,
    resetTable: result.ok,
  } as const;
}

export function transitionLocalDateTimeFilterState<T extends LocalDateTimeFilterDraft>(
  currentState: LocalDateTimeFilterState<T>,
  nextDraft: T
): LocalDateTimeFilterTransition<T> {
  const transition = updateLocalDateTimeFilterState(currentState.query, nextDraft);
  return {
    state: {
      draft: transition.draft,
      query: transition.query,
      error: transition.error,
    },
    resetTable: transition.resetTable,
  };
}

export function toUtcLocalDateTimeFilterQuery(
  draft: LocalDateTimeFilterDraft
): LocalDateTimeFilterQueryResult {
  const dates = parseLocalDateTimeRange(draft.begin_time, draft.end_time);
  if (!dates.ok) return dates;
  return {
    ok: true,
    query: {
      ...draft,
      begin_time: dates.begin?.toISOString() ?? '',
      end_time: dates.end?.toISOString() ?? '',
    },
  };
}

function parseLocalDateTime(value: string) {
  const match = LOCAL_DATE_TIME_PATTERN.exec(value);
  if (!match) return null;
  const parts = dateParts(match);
  const parsed = new Date(0);
  parsed.setFullYear(parts.year, parts.month - MONTH_INDEX_OFFSET, parts.day);
  parsed.setHours(parts.hour, parts.minute, parts.second, parts.millisecond);
  return dateMatches(parsed, parts) ? parsed : null;
}

function dateMatches(parsed: Date, parts: ReturnType<typeof dateParts>) {
  return (
    !Number.isNaN(parsed.getTime()) &&
    parsed.getFullYear() === parts.year &&
    parsed.getMonth() === parts.month - MONTH_INDEX_OFFSET &&
    parsed.getDate() === parts.day &&
    parsed.getHours() === parts.hour &&
    parsed.getMinutes() === parts.minute &&
    parsed.getSeconds() === parts.second &&
    parsed.getMilliseconds() === parts.millisecond
  );
}

function dateParts(match: RegExpExecArray) {
  return {
    year: Number(match[DATE_PART.YEAR]),
    month: Number(match[DATE_PART.MONTH]),
    day: Number(match[DATE_PART.DAY]),
    hour: Number(match[DATE_PART.HOUR]),
    minute: Number(match[DATE_PART.MINUTE]),
    second: Number(match[DATE_PART.SECOND] ?? 0),
    millisecond: Number((match[DATE_PART.MILLISECOND] ?? '').padEnd(3, '0') || 0),
  };
}
