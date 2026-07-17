'use client';

import type {
  AuditFilterError,
  OperationLogSummary,
  OperationLogSortField,
  OperationLogFilterQuery,
} from 'src/entities/audit-log';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { usePendingMutation } from 'src/shared/api/use-pending-mutation';

import { usePermissionChecker } from 'src/entities/session';
import {
  useOperationLogs,
  isAuditSortField,
  AUDIT_FILTER_ERROR,
  useOperationLogDetail,
  OPERATION_LOG_SORT_FIELDS,
  createOperationLogListQuery,
  DEFAULT_OPERATION_LOG_QUERY,
  updateOperationLogFilterState,
  DEFAULT_OPERATION_LOG_FILTERS,
} from 'src/entities/audit-log';

import { auditLogCapabilities } from './permissions';
import { resetAuditTableSort, resetAuditMutationCursor } from './table-actions';
import {
  deleteOperationLog,
  cleanOperationLogs,
  deleteOperationLogs,
  exportOperationLogs,
} from '../api';

const FILTER_ERROR_KEYS = {
  [AUDIT_FILTER_ERROR.INVALID_DATE_TIME]: 'filters.invalidDateTime',
  [AUDIT_FILTER_ERROR.INVALID_RANGE]: 'filters.invalidRange',
} as const;
const OPERATION_DELETE_TARGET_REQUIRED_ERROR = 'An operation log delete target is required';
const OPERATION_EXPORT_FILTERS_REQUIRED_ERROR =
  'Valid operation-log filters are required for export';

export function useOperationLogController() {
  const state = useOperationLogState();
  const resources = useOperationLogResources(state);
  const mutation = usePendingMutation();
  const actions = useOperationLogActions({ state, resources, mutation });
  return { state, resources, actions, pending: mutation.pending };
}

export type OperationLogController = ReturnType<typeof useOperationLogController>;

function useOperationLogState() {
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'oper_time',
    defaultOrder: 'desc',
  });
  const [filterDraft, setFilterDraft] = useState(DEFAULT_OPERATION_LOG_FILTERS);
  const [filterQuery, setFilterQuery] = useState<OperationLogFilterQuery>(
    DEFAULT_OPERATION_LOG_QUERY
  );
  const [filterError, setFilterError] = useState<AuditFilterError | null>(null);
  const [detailTarget, setDetailTarget] = useState<OperationLogSummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<OperationLogSummary | null>(null);
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
    detailTarget,
    setDetailTarget,
    deleteTarget,
    setDeleteTarget,
    batchOpen,
    setBatchOpen,
    cleanOpen,
    setCleanOpen,
  };
}

function useOperationLogResources(state: ReturnType<typeof useOperationLogState>) {
  const { t } = useTranslate('audit');
  const permissions = auditLogCapabilities(usePermissionChecker()).operation;
  const listQuery = useMemo(
    () =>
      createOperationLogListQuery(state.filterQuery, {
        sort_by: operationSortField(state.table.orderBy),
        sort_order: state.table.order,
      }),
    [state.filterQuery, state.table.order, state.table.orderBy]
  );
  const logs = useOperationLogs({
    cursor: state.table.cursorRequest,
    query: listQuery,
    enabled: permissions.list,
  });
  const detail = useOperationLogDetail(state.detailTarget?.oper_id ?? null, permissions.query);
  return {
    t,
    logs,
    detail,
    listQuery,
    canList: permissions.list,
    canQuery: permissions.query,
    canRemove: permissions.remove,
    canExport: permissions.export,
    filtersValid: state.filterError === null,
    filterErrorMessage: state.filterError ? t(FILTER_ERROR_KEYS[state.filterError]) : null,
  };
}

type OperationActionOptions = Readonly<{
  state: ReturnType<typeof useOperationLogState>;
  resources: ReturnType<typeof useOperationLogResources>;
  mutation: ReturnType<typeof usePendingMutation>;
}>;

function useOperationLogActions(options: OperationActionOptions) {
  const changeFilters = useOperationFilterAction(options.state);
  const resetFilters = useCallback(() => {
    changeFilters(DEFAULT_OPERATION_LOG_FILTERS);
    resetAuditTableSort(options.state.table, 'oper_time');
  }, [changeFilters, options.state.table]);
  const sort = useOperationSortAction(options.state);
  const deletes = useOperationDeleteActions(options);
  const maintenance = useOperationMaintenanceActions(options);
  return {
    ...deletes,
    ...maintenance,
    sort,
    changeFilters,
    resetFilters,
    openDetail: options.state.setDetailTarget,
    closeDetail: () => options.state.setDetailTarget(null),
  };
}

function useOperationFilterAction(state: OperationActionOptions['state']) {
  return useCallback(
    (draft: typeof state.filterDraft) => {
      const next = updateOperationLogFilterState(state.filterQuery, draft);
      state.setFilterDraft(next.draft);
      state.setFilterError(next.error);
      state.setFilterQuery(next.query);
      if (!next.resetTable) return;
      state.table.onResetCursor();
    },
    [state]
  );
}

function useOperationSortAction(state: OperationActionOptions['state']) {
  return useCallback(
    (field: string) => {
      if (!isAuditSortField(field, OPERATION_LOG_SORT_FIELDS)) return;
      state.table.onSort(field);
    },
    [state]
  );
}

function useOperationDeleteActions(options: OperationActionOptions) {
  const confirmDelete = useCallback(() => {
    const target = options.state.deleteTarget;
    if (!target) throw new Error(OPERATION_DELETE_TARGET_REQUIRED_ERROR);
    return options.mutation.run({
      key: `delete:${target.oper_id}`,
      failureMessage: options.resources.t('messages.deleteFailure'),
      action: () => deleteOperationLog(target.oper_id),
      onSuccess: () => finishSingleDelete(options),
    });
  }, [options]);
  const confirmBatchDelete = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:batch',
        failureMessage: options.resources.t('messages.deleteFailure'),
        action: () => deleteOperationLogs(options.state.table.selected),
        onSuccess: () => finishBatchDelete(options),
      }),
    [options]
  );
  return { confirmDelete, confirmBatchDelete };
}

function useOperationMaintenanceActions(options: OperationActionOptions) {
  const confirmClean = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:clean',
        failureMessage: options.resources.t('messages.cleanFailure'),
        action: cleanOperationLogs,
        onSuccess: () => finishClean(options),
      }),
    [options]
  );
  const submitExport = useCallback(() => {
    if (!options.resources.filtersValid) {
      throw new Error(OPERATION_EXPORT_FILTERS_REQUIRED_ERROR);
    }
    return options.mutation.run({
      key: 'export',
      failureMessage: options.resources.t('messages.exportFailure'),
      action: () => exportOperationLogs(options.resources.listQuery),
    });
  }, [options]);
  return { confirmClean, submitExport };
}

function finishSingleDelete(options: OperationActionOptions) {
  resetAuditMutationCursor(options.state.table);
  options.state.setDeleteTarget(null);
  toast.success(options.resources.t('messages.deleteSuccess'));
}

function finishBatchDelete(options: OperationActionOptions) {
  resetAuditMutationCursor(options.state.table);
  options.state.setBatchOpen(false);
  toast.success(options.resources.t('messages.deleteSuccess'));
}

function finishClean(options: OperationActionOptions) {
  options.state.table.onResetCursor();
  options.state.setCleanOpen(false);
  toast.success(options.resources.t('messages.cleanSuccess'));
}

function operationSortField(value: string): OperationLogSortField {
  if (isAuditSortField(value, OPERATION_LOG_SORT_FIELDS)) return value;
  throw new Error(`Unsupported operation-log sort field: ${value}`);
}
