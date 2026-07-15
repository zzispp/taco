import { useMemo } from 'react';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useHasPermission } from 'src/entities/session';
import { useUsers, useUserFormOptions } from 'src/entities/user';
import {
  usePublicConfigs,
  PUBLIC_CONFIG_KEYS,
  passwordPolicyFromPublicConfigs,
} from 'src/entities/system';

import { withSelectionHead } from 'src/widgets/admin-common';

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
  const canAdd = useHasPermission('system:user:add');
  const canDelete = useHasPermission('system:user:remove');
  const canImport = useHasPermission('system:user:import');
  const canExport = useHasPermission('system:user:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableUsers = users.items;

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
  };
}
