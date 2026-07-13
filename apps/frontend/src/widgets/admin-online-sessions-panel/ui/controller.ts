import type {
  OnlineSession,
  OnlineSessionQuery,
  OnlineSessionFilters,
  OnlineSessionFilterError,
} from 'src/entities/online-session';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import {
  useOnlineSessions,
  ONLINE_SESSION_FILTER_ERROR,
  DEFAULT_ONLINE_SESSION_QUERY,
  updateOnlineSessionFilterState,
  DEFAULT_ONLINE_SESSION_FILTERS,
} from 'src/entities/online-session';

import { forceLogoutOnlineSession } from 'src/features/online-session-management';

import { pageRows, onlineSessionHead } from './helpers';

const FILTER_ERROR_TRANSLATION_KEY = {
  [ONLINE_SESSION_FILTER_ERROR.INVALID_DATE_TIME]: 'onlineSessions.invalidDateTime',
  [ONLINE_SESSION_FILTER_ERROR.INVALID_RANGE]: 'onlineSessions.invalidTimeRange',
} as const;

export function useOnlineSessionsController() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFiltersState] = useState(DEFAULT_ONLINE_SESSION_FILTERS);
  const [query, setQuery] = useState(DEFAULT_ONLINE_SESSION_QUERY);
  const [filterError, setFilterError] = useState<OnlineSessionFilterError | null>(null);
  const [forceTarget, setForceTarget] = useState<OnlineSession | null>(null);
  const sessions = useOnlineSessions(query);
  const canForceLogout = useHasPermission('system:online:forceLogout');
  const head = useMemo(() => onlineSessionHead(t), [t]);
  const rows = useMemo(
    () => pageRows(sessions.rows, table.page, table.rowsPerPage),
    [sessions.rows, table.page, table.rowsPerPage]
  );
  const setFilters = useFilterWriter({
    query,
    setQuery,
    setFilterError,
    setFiltersState,
    resetPage: table.onResetPage,
  });
  const confirmForceLogout = useForceLogoutAction(forceTarget, setForceTarget, t);
  const filterErrorMessage = filterError ? t(FILTER_ERROR_TRANSLATION_KEY[filterError]) : null;

  return {
    resources: {
      t,
      table,
      filters,
      sessions,
      rows,
      head,
      canForceLogout,
      filterErrorMessage,
    },
    state: { forceTarget, setForceTarget },
    actions: { setFilters, confirmForceLogout },
  };
}

function useFilterWriter(options: FilterWriterOptions) {
  const { query, setQuery, setFilterError, setFiltersState, resetPage } = options;
  return useCallback(
    (next: OnlineSessionFilters) => {
      const transition = updateOnlineSessionFilterState(query, next);
      setFiltersState(transition.draft);
      setFilterError(transition.error);
      if (!transition.resetTable) return;
      resetPage();
      setQuery(transition.query);
    },
    [query, resetPage, setFilterError, setFiltersState, setQuery]
  );
}

function useForceLogoutAction(
  target: OnlineSession | null,
  setTarget: (target: OnlineSession | null) => void,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (!target) return;
    try {
      await forceLogoutOnlineSession(target.tokenId);
      toast.success(t('onlineSessions.forceLogoutSuccess'));
      setTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [setTarget, t, target]);
}

type FilterWriterOptions = Readonly<{
  query: OnlineSessionQuery;
  setQuery: (query: OnlineSessionQuery) => void;
  setFilterError: (error: OnlineSessionFilterError | null) => void;
  setFiltersState: (filters: OnlineSessionFilters) => void;
  resetPage: () => void;
}>;
