import type { OnlineSession, OnlineSessionFilters } from 'src/entities/online-session';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import { useOnlineSessions } from 'src/entities/online-session';

import { forceLogoutOnlineSession } from 'src/features/online-session-management';

import { DEFAULT_FILTERS } from './constants';
import { pageRows, onlineSessionHead } from './helpers';

export function useOnlineSessionsController() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFiltersState] = useState(DEFAULT_FILTERS);
  const [forceTarget, setForceTarget] = useState<OnlineSession | null>(null);
  const sessions = useOnlineSessions(filters);
  const canForceLogout = useHasPermission('system:online:forceLogout');
  const head = useMemo(() => onlineSessionHead(t), [t]);
  const rows = useMemo(
    () => pageRows(sessions.rows, table.page, table.rowsPerPage),
    [sessions.rows, table.page, table.rowsPerPage]
  );
  const setFilters = useFilterWriter(setFiltersState, table.onResetPage);
  const confirmForceLogout = useForceLogoutAction(forceTarget, setForceTarget, t);

  return {
    resources: { t, table, filters, sessions, rows, head, canForceLogout },
    state: { forceTarget, setForceTarget },
    actions: { setFilters, confirmForceLogout },
  };
}

function useFilterWriter(
  setFiltersState: (filters: OnlineSessionFilters) => void,
  resetPage: () => void
) {
  return useCallback((next: OnlineSessionFilters) => {
    resetPage();
    setFiltersState(next);
  }, [resetPage, setFiltersState]);
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
