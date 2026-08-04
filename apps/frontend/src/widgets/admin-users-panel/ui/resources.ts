import { useMemo, useCallback } from 'react';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { withSelectionHead } from 'src/shared/ui/admin-common';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { refreshCursorPage } from 'src/shared/api/refresh-cursor-page';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useUsers } from 'src/entities/user';
import { useHasPermission } from 'src/entities/session';
import {
  usePublicConfigs,
  PUBLIC_CONFIG_KEYS,
  passwordPolicyFromPublicConfigs,
} from 'src/entities/system';

import { useUserFormOptions } from 'src/features/user-management';

import { DEFAULT_FILTERS } from './constants';
import { userHead, flattenDeptNames } from './helpers';

export function useUserResources() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const filters = useLocalDateTimeFilterState(DEFAULT_FILTERS, {
    onValidQuery: table.onResetCursor,
  });
  const users = useUsers(table.cursorRequest, filters.query);
  const options = useUserFormOptions();
  const publicConfigs = usePublicConfigs([PUBLIC_CONFIG_KEYS.passwordPolicy]);
  const passwordPolicy = useMemo(
    () => passwordPolicyFromPublicConfigs(publicConfigs.data),
    [publicConfigs.data]
  );
  const roles = useMemo(() => options.data?.roles ?? [], [options.data?.roles]);
  const posts = useMemo(() => options.data?.posts ?? [], [options.data?.posts]);
  const deptTree = useMemo(() => options.data?.depts ?? [], [options.data?.depts]);
  const depts = useMemo(() => flattenDeptNames(deptTree), [deptTree]);
  const head = useMemo(() => userHead(t), [t]);
  const permissions = useUserPermissions();
  const { canAdd, canDelete, canImport, canExport } = permissions;
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableUsers = users.items;
  const pageRefresh = useUserPageRefresh({ table, users, formOptions: options, publicConfigs });

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
    passwordPolicy,
    ...pageRefresh,
  };
}

function useUserPermissions() {
  const canAdd = useHasPermission('system:user:add');
  const canDelete = useHasPermission('system:user:remove');
  const canImport = useHasPermission('system:user:import');
  const canExport = useHasPermission('system:user:export');

  return { canAdd, canDelete, canImport, canExport };
}

function useUserPageRefresh(
  options: Readonly<{
    table: ReturnType<typeof useTable>;
    users: ReturnType<typeof useUsers>;
    formOptions: ReturnType<typeof useUserFormOptions>;
    publicConfigs: ReturnType<typeof usePublicConfigs>;
  }>
) {
  const refreshPage = useCallback(async () => {
    await Promise.all([
      refreshCursorPage({
        cursor: options.table.cursor,
        resetCursor: options.table.onResetCursor,
        refresh: options.users.refresh,
      }),
      options.formOptions.mutate(),
      options.publicConfigs.mutate(),
    ]);
  }, [options]);
  const refreshing =
    options.users.isValidating ||
    options.formOptions.isValidating ||
    options.publicConfigs.isValidating;
  return { refreshPage, refreshing };
}
