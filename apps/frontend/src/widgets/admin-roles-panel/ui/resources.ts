import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

import { useMemo, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import { useRoles } from 'src/entities/role';
import { useHasPermission } from 'src/entities/session';

import { exportRoles } from 'src/features/role-management';

import { withSelectionHead } from 'src/widgets/admin-common';

import { roleHead } from './helpers';
import { DEFAULT_FILTERS } from './constants';

export function useRoleResources() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const filters = useLocalDateTimeFilterState(DEFAULT_FILTERS, {
    onValidQuery: table.onResetCursor,
  });
  const roles = useRoles(table.cursorRequest, filters.query);
  const head = useMemo(() => roleHead(t), [t]);
  const canAdd = useHasPermission('system:role:add');
  const canDelete = useHasPermission('system:role:remove');
  const canExport = useHasPermission('system:role:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableRoles = useMemo(() => roles.items.filter((role) => !role.system), [roles.items]);

  return {
    t,
    table,
    filters: filters.draft,
    setFilters: filters.change,
    filterQuery: filters.query,
    filterError: filters.error,
    roles,
    head,
    canAdd,
    canDelete,
    canExport,
    loadingHead,
    selectableRoles,
  };
}

export function useRoleExportAction({ filters, filterError, t }: RoleExportOptions) {
  const submitExport = useCallback(async () => {
    if (filterError) {
      toast.error(t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[filterError]));
      return;
    }
    try {
      await exportRoles(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [filterError, filters, t]);

  return { submitExport };
}

type RoleExportOptions = Readonly<{
  filters: Readonly<Record<string, string>>;
  filterError: LocalDateTimeFilterError | null;
  t: ReturnType<typeof useTranslate>['t'];
}>;
