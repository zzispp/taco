import type {
  SystemLogFilters,
  SystemLogFilterError,
  SystemLogFilterQuery,
} from 'src/entities/system-log';

import { toSystemLogQuery, hasRequiredSystemLogRange } from 'src/entities/system-log';

export type SystemLogFilterTransition = Readonly<{
  query: SystemLogFilterQuery;
  error: SystemLogFilterError | null;
  resetTable: boolean;
}>;

export type SystemLogActionFilter =
  | Readonly<{ kind: 'valid'; query: SystemLogFilterQuery }>
  | Readonly<{ kind: 'invalid'; error: SystemLogFilterError }>
  | Readonly<{ kind: 'missing_range' }>;

export function applySystemLogFilterDraft(
  currentQuery: SystemLogFilterQuery,
  draft: SystemLogFilters
): SystemLogFilterTransition {
  const result = toSystemLogQuery(draft);
  if (!result.ok) {
    return { query: currentQuery, error: result.error, resetTable: false };
  }
  return { query: result.query, error: null, resetTable: true };
}

export function resolveSystemLogActionFilter(draft: SystemLogFilters): SystemLogActionFilter {
  const result = toSystemLogQuery(draft);
  if (!result.ok) return { kind: 'invalid', error: result.error };
  if (!hasRequiredSystemLogRange(result.query)) return { kind: 'missing_range' };
  return { kind: 'valid', query: result.query };
}
