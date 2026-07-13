import { useMemo } from 'react';

import { useTable } from 'src/shared/ui/table';
import { withSelectionHead } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useHasPermission } from 'src/entities/session';
import { useUsers, useUserFormOptions } from 'src/entities/user';

import { DEFAULT_FILTERS } from './constants';
import { userHead, flattenDeptNames } from './helpers';

export function useUserResources() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const filters = useLocalDateTimeFilterState(DEFAULT_FILTERS, {
    onValidQuery: table.onResetPage,
  });
  const users = useUsers(table.page, table.rowsPerPage, filters.query);
  const options = useUserFormOptions();
  const roles = useMemo(() => options.data?.roles ?? [], [options.data?.roles]);
  const posts = useMemo(() => options.data?.posts ?? [], [options.data?.posts]);
  const deptTree = useMemo(() => options.data?.depts ?? [], [options.data?.depts]);
  const depts = useMemo(() => flattenDeptNames(deptTree), [deptTree]);
  const head = useMemo(() => userHead(t), [t]);
  const canAdd = useHasPermission('system:user:add');
  const canDelete = useHasPermission('system:user:remove');
  const canImport = useHasPermission('system:user:import');
  const canExport = useHasPermission('system:user:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableUsers = useMemo(() => users.items.filter((user) => !user.system), [users.items]);

  return {
    t,
    table,
    filters: filters.draft,
    setFilters: filters.change,
    filterQuery: filters.query,
    filterError: filters.error,
    users,
    roles,
    posts,
    deptTree,
    depts,
    head,
    canAdd,
    canDelete,
    canImport,
    canExport,
    loadingHead,
    selectableUsers,
  };
}
