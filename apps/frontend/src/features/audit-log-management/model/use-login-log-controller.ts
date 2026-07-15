'use client';

import type {
  LoginLog,
  AuditFilterError,
  LoginLogSortField,
  LoginLogFilterQuery,
} from 'src/entities/audit-log';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { usePermissionChecker } from 'src/entities/session';
import {
  useLoginLogs,
  isAuditSortField,
  AUDIT_FILTER_ERROR,
  LOGIN_LOG_SORT_FIELDS,
  createLoginLogListQuery,
  DEFAULT_LOGIN_LOG_QUERY,
  updateLoginLogFilterState,
  DEFAULT_LOGIN_LOG_FILTERS,
} from 'src/entities/audit-log';

import { useAuditMutation } from './use-audit-mutation';
import { auditLogCapabilities, loginLogSelectionActions } from './permissions';
import { resetAuditTableSort, resetAuditMutationCursor } from './table-actions';
import {
  deleteLoginLog,
  cleanLoginLogs,
  deleteLoginLogs,
  exportLoginLogs,
  unlockLoginAccount,
} from '../api';

const FILTER_ERROR_KEYS = {
  [AUDIT_FILTER_ERROR.INVALID_DATE_TIME]: 'filters.invalidDateTime',
  [AUDIT_FILTER_ERROR.INVALID_RANGE]: 'filters.invalidRange',
} as const;
const LOGIN_DELETE_TARGET_REQUIRED_ERROR = 'A login log delete target is required';
const LOGIN_UNLOCK_TARGET_REQUIRED_ERROR = 'A login log unlock target is required';
const LOGIN_EXPORT_FILTERS_REQUIRED_ERROR = 'Valid login-log filters are required for export';

export function useLoginLogController() {
  const state = useLoginLogState();
  const resources = useLoginLogResources(state);
  const mutation = useAuditMutation();
  const actions = useLoginLogActions({ state, resources, mutation });
  return { state, resources, actions, pending: mutation.pending };
}

export type LoginLogController = ReturnType<typeof useLoginLogController>;

function useLoginLogState() {
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'login_time',
    defaultOrder: 'desc',
  });
  const [filterDraft, setFilterDraft] = useState(DEFAULT_LOGIN_LOG_FILTERS);
  const [filterQuery, setFilterQuery] = useState<LoginLogFilterQuery>(DEFAULT_LOGIN_LOG_QUERY);
  const [filterError, setFilterError] = useState<AuditFilterError | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<LoginLog | null>(null);
  const [unlockTarget, setUnlockTarget] = useState<LoginLog | null>(null);
  const [batchOpen, setBatchOpen] = useState(false);
  const [cleanOpen, setCleanOpen] = useState(false);
  return {
    table,
    filterDraft,
    setFilterDraft,
    filterQuery,
    setFilterQuery,
    filterError,
    setFilterError,
    deleteTarget,
    setDeleteTarget,
    unlockTarget,
    setUnlockTarget,
    batchOpen,
    setBatchOpen,
    cleanOpen,
    setCleanOpen,
  };
}

function useLoginLogResources(state: ReturnType<typeof useLoginLogState>) {
  const { t } = useTranslate('audit');
  const permissions = auditLogCapabilities(usePermissionChecker()).login;
  const listQuery = useMemo(
    () =>
      createLoginLogListQuery(state.filterQuery, {
        sort_by: loginSortField(state.table.orderBy),
        sort_order: state.table.order,
      }),
    [state.filterQuery, state.table.order, state.table.orderBy]
  );
  const logs = useLoginLogs({
    cursor: state.table.cursorRequest,
    query: listQuery,
    enabled: permissions.list,
  });
  const selection = loginLogSelectionActions(
    state.table.selected,
    permissions.remove,
    permissions.unlock
  );
  return {
    t,
    logs,
    listQuery,
    selection,
    canList: permissions.list,
    canRemove: permissions.remove,
    canExport: permissions.export,
    canUnlock: permissions.unlock,
    filtersValid: state.filterError === null,
    filterErrorMessage: state.filterError ? t(FILTER_ERROR_KEYS[state.filterError]) : null,
  };
}

type LoginActionOptions = Readonly<{
  state: ReturnType<typeof useLoginLogState>;
  resources: ReturnType<typeof useLoginLogResources>;
  mutation: ReturnType<typeof useAuditMutation>;
}>;

function useLoginLogActions(options: LoginActionOptions) {
  const changeFilters = useLoginFilterAction(options.state);
  const resetFilters = useCallback(() => {
    changeFilters(DEFAULT_LOGIN_LOG_FILTERS);
    resetAuditTableSort(options.state.table, 'login_time');
  }, [changeFilters, options.state.table]);
  const sort = useLoginSortAction(options.state);
  const deletes = useLoginDeleteActions(options);
  const maintenance = useLoginMaintenanceActions(options);
  return { ...deletes, ...maintenance, sort, changeFilters, resetFilters };
}

function useLoginFilterAction(state: LoginActionOptions['state']) {
  return useCallback(
    (draft: typeof state.filterDraft) => {
      const next = updateLoginLogFilterState(state.filterQuery, draft);
      state.setFilterDraft(next.draft);
      state.setFilterError(next.error);
      state.setFilterQuery(next.query);
      if (!next.resetTable) return;
      state.table.onResetCursor();
    },
    [state]
  );
}

function useLoginSortAction(state: LoginActionOptions['state']) {
  return useCallback(
    (field: string) => {
      if (!isAuditSortField(field, LOGIN_LOG_SORT_FIELDS)) return;
      state.table.onSort(field);
    },
    [state]
  );
}

function useLoginDeleteActions(options: LoginActionOptions) {
  const confirmDelete = useCallback(() => {
    const target = options.state.deleteTarget;
    if (!target) throw new Error(LOGIN_DELETE_TARGET_REQUIRED_ERROR);
    return options.mutation.run({
      key: `delete:${target.info_id}`,
      failureMessage: options.resources.t('messages.deleteFailure'),
      action: () => deleteLoginLog(target.info_id),
      onSuccess: () => finishSingleDelete(options),
    });
  }, [options]);
  const confirmBatchDelete = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:batch',
        failureMessage: options.resources.t('messages.deleteFailure'),
        action: () => deleteLoginLogs(options.state.table.selected),
        onSuccess: () => finishBatchDelete(options),
      }),
    [options]
  );
  return { confirmDelete, confirmBatchDelete };
}

function useLoginMaintenanceActions(options: LoginActionOptions) {
  const requestUnlock = useCallback(() => {
    const [selectedId] = options.state.table.selected;
    const target = options.resources.logs.items.find((item) => item.info_id === selectedId);
    if (target && options.resources.selection.canUnlock) options.state.setUnlockTarget(target);
  }, [options]);
  const confirmUnlock = useCallback(() => {
    const target = options.state.unlockTarget;
    if (!target) throw new Error(LOGIN_UNLOCK_TARGET_REQUIRED_ERROR);
    return options.mutation.run({
      key: 'unlock',
      failureMessage: options.resources.t('messages.unlockFailure'),
      action: () => unlockLoginAccount(target.user_name),
      onSuccess: () => finishUnlock(options),
    });
  }, [options]);
  const confirmClean = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:clean',
        failureMessage: options.resources.t('messages.cleanFailure'),
        action: cleanLoginLogs,
        onSuccess: () => finishClean(options),
      }),
    [options]
  );
  const submitExport = useCallback(() => {
    if (!options.resources.filtersValid) {
      throw new Error(LOGIN_EXPORT_FILTERS_REQUIRED_ERROR);
    }
    return options.mutation.run({
      key: 'export',
      failureMessage: options.resources.t('messages.exportFailure'),
      action: () => exportLoginLogs(options.resources.listQuery),
    });
  }, [options]);
  return { requestUnlock, confirmUnlock, confirmClean, submitExport };
}

function finishSingleDelete(options: LoginActionOptions) {
  resetAuditMutationCursor(options.state.table);
  options.state.setDeleteTarget(null);
  toast.success(options.resources.t('messages.deleteSuccess'));
}

function finishBatchDelete(options: LoginActionOptions) {
  resetAuditMutationCursor(options.state.table);
  options.state.setBatchOpen(false);
  toast.success(options.resources.t('messages.deleteSuccess'));
}

function finishClean(options: LoginActionOptions) {
  options.state.table.onResetCursor();
  options.state.setCleanOpen(false);
  toast.success(options.resources.t('messages.cleanSuccess'));
}

function finishUnlock(options: LoginActionOptions) {
  options.state.table.onResetCursor();
  options.state.setUnlockTarget(null);
  toast.success(options.resources.t('messages.unlockSuccess'));
}

function loginSortField(value: string): LoginLogSortField {
  if (isAuditSortField(value, LOGIN_LOG_SORT_FIELDS)) return value;
  throw new Error(`Unsupported login-log sort field: ${value}`);
}
