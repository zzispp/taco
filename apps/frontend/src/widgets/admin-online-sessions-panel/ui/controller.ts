import type {
  OnlineSession,
  OnlineSessionQuery,
  OnlineSessionFilters,
  OnlineSessionFilterError,
} from 'src/entities/online-session';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { useHasPermission } from 'src/entities/session';
import {
  useOnlineSessions,
  ONLINE_SESSION_FILTER_ERROR,
  DEFAULT_ONLINE_SESSION_QUERY,
  updateOnlineSessionFilterState,
  DEFAULT_ONLINE_SESSION_FILTERS,
} from 'src/entities/online-session';

import { forceLogoutOnlineSession } from 'src/features/online-session-management';

import { onlineSessionHead } from './helpers';

const FILTER_ERROR_TRANSLATION_KEY = {
  [ONLINE_SESSION_FILTER_ERROR.INVALID_DATE_TIME]: 'onlineSessions.invalidDateTime',
  [ONLINE_SESSION_FILTER_ERROR.INVALID_RANGE]: 'onlineSessions.invalidTimeRange',
} as const;

export function useOnlineSessionsController() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const [filters, setFiltersState] = useState(DEFAULT_ONLINE_SESSION_FILTERS);
  const [query, setQuery] = useState(DEFAULT_ONLINE_SESSION_QUERY);
  const [filterError, setFilterError] = useState<OnlineSessionFilterError | null>(null);
  const [forceTarget, setForceTarget] = useState<OnlineSession | null>(null);
  const sessions = useOnlineSessions(table.cursorRequest, query);
  const canForceLogout = useHasPermission('system:online:forceLogout');
  const head = useMemo(() => onlineSessionHead(t), [t]);
  const setFilters = useFilterWriter({
    query,
    setQuery,
    setFilterError,
    setFiltersState,
    resetCursor: table.onResetCursor,
  });
  const confirmForceLogout = useForceLogoutAction({
    target: forceTarget,
    setTarget: setForceTarget,
    resetCursor: table.onResetCursor,
    t,
  });
  const filterErrorMessage = filterError ? t(FILTER_ERROR_TRANSLATION_KEY[filterError]) : null;

  return {
    resources: {
      t,
      table,
      filters,
      sessions,
      rows: sessions.rows,
      head,
      canForceLogout,
      filterErrorMessage,
    },
    state: { forceTarget, setForceTarget },
    actions: { setFilters, confirmForceLogout },
  };
}

function useFilterWriter(options: FilterWriterOptions) {
  const { query, setQuery, setFilterError, setFiltersState, resetCursor } = options;
  return useCallback(
    (next: OnlineSessionFilters) => {
      const transition = updateOnlineSessionFilterState(query, next);
      setFiltersState(transition.draft);
      setFilterError(transition.error);
      if (!transition.resetTable) return;
      resetCursor();
      setQuery(transition.query);
    },
    [query, resetCursor, setFilterError, setFiltersState, setQuery]
  );
}

function useForceLogoutAction({ target, setTarget, resetCursor, t }: ForceLogoutOptions) {
  return useCallback(async () => {
    if (!target) return;
    try {
      await forceLogoutOnlineSession(target.tokenId);
      toast.success(t('onlineSessions.forceLogoutSuccess'));
      setTarget(null);
      resetCursor();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [resetCursor, setTarget, t, target]);
}

type FilterWriterOptions = Readonly<{
  query: OnlineSessionQuery;
  setQuery: (query: OnlineSessionQuery) => void;
  setFilterError: (error: OnlineSessionFilterError | null) => void;
  setFiltersState: (filters: OnlineSessionFilters) => void;
  resetCursor: () => void;
}>;

type ForceLogoutOptions = Readonly<{
  target: OnlineSession | null;
  setTarget: (target: OnlineSession | null) => void;
  resetCursor: () => void;
  t: ReturnType<typeof useTranslate>['t'];
}>;
