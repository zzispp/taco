'use client';

import type { SystemLogFilters, SystemLogFilterError } from 'src/entities/system-log';

import { useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { apiMutationErrorMessage } from 'src/shared/api/mutation-error';
import { usePendingMutation } from 'src/shared/api/use-pending-mutation';

import { usePermissionChecker } from 'src/entities/session';
import {
  useSystemLogs,
  toSystemLogQuery,
  useSystemLogDetail,
  SYSTEM_LOG_FILTER_ERROR,
  useSystemLogCleanupExecution,
  createDefaultSystemLogFilters,
} from 'src/entities/system-log';

import { systemLogCapabilities } from './permissions';
import { useCleanupExecutionNotification } from './cleanup-execution';
import { useSystemLogState, type SystemLogState } from './use-system-log-state';
import { applySystemLogFilterDraft, resolveSystemLogActionFilter } from './filter-state';
import { createSystemLogCleanupPreview, acceptsSystemLogCleanupPreview } from './cleanup-preview';
import {
  cleanSystemLogs,
  deleteSystemLog,
  deleteSystemLogs,
  exportSystemLogs,
  refreshSystemLogs,
  countSystemLogsForCleanup,
} from '../api';

const FILTER_ERROR_KEYS = {
  [SYSTEM_LOG_FILTER_ERROR.INVALID_DATE_TIME]: 'filters.invalidDateTime',
  [SYSTEM_LOG_FILTER_ERROR.INVALID_RANGE]: 'filters.invalidRange',
} as const;

export function useSystemLogController() {
  const state = useSystemLogState();
  const resources = useSystemLogResources(state);
  const mutation = usePendingMutation();
  const actions = useSystemLogActions({ state, resources, mutation });
  useCleanupExecutionNotification({
    execution: resources.cleanupExecution.data,
    onTerminal: refreshSystemLogs,
    onClear: state.clearCleanupExecution,
    t: resources.t,
  });
  return { state, resources, actions, pending: mutation.pending };
}

export type SystemLogController = ReturnType<typeof useSystemLogController>;

function useSystemLogResources(state: SystemLogState) {
  const { t } = useTranslate('systemLog');
  const permissions = systemLogCapabilities(usePermissionChecker());
  const logs = useSystemLogs({
    cursor: state.table.cursorRequest,
    query: state.filterQuery,
    enabled: permissions.list,
  });
  const detail = useSystemLogDetail(state.detailTarget?.log_id ?? null, permissions.query);
  const cleanupExecution = useSystemLogCleanupExecution(
    state.cleanupExecutionId,
    permissions.remove
  );
  return {
    t,
    logs,
    detail,
    cleanupExecution,
    canList: permissions.list,
    canQuery: permissions.query,
    canRemove: permissions.remove,
    canExport: permissions.export,
    filtersValid: state.filterError === null,
    hasRequiredRange: Boolean(state.filterQuery.begin_time && state.filterQuery.end_time),
    filterErrorMessage: state.filterError ? t(FILTER_ERROR_KEYS[state.filterError]) : null,
    listErrorMessage: resourcesErrorMessage(logs.error, t('messages.listFailure')),
  };
}

type Options = Readonly<{
  state: SystemLogState;
  resources: ReturnType<typeof useSystemLogResources>;
  mutation: ReturnType<typeof usePendingMutation>;
}>;

function useSystemLogActions(options: Options) {
  const changeFilters = useSystemLogFilterAction(options.state);
  const applyFilters = useSystemLogApplyFilterAction(options.state);
  const resetFilters = useCallback(
    () => applySystemLogFilters(options.state, createDefaultSystemLogFilters()),
    [options.state]
  );
  return {
    changeFilters,
    applyFilters,
    resetFilters,
    openDetail: options.state.setDetailTarget,
    closeDetail: () => options.state.setDetailTarget(null),
    requestDelete: options.state.setDeleteTarget,
    confirmDelete: () => confirmDelete(options),
    confirmBatchDelete: () => confirmBatchDelete(options),
    requestClean: () => requestClean(options),
    cancelClean: () => invalidateCleanupPreview(options.state),
    confirmClean: () => confirmClean(options),
    dismissCleanupExecution: options.state.dismissCleanupExecution,
    showCleanupExecution: options.state.showCleanupExecution,
    submitExport: () => submitExport(options),
  };
}

function useSystemLogFilterAction(state: SystemLogState) {
  return useCallback(
    (draft: SystemLogFilters) => {
      state.setFilterDraft(draft);
      state.setFilterError(filterDraftError(draft));
      invalidateCleanupPreview(state);
    },
    [state]
  );
}

function useSystemLogApplyFilterAction(state: SystemLogState) {
  return useCallback(() => applySystemLogFilters(state, state.filterDraft), [state]);
}

function confirmDelete(options: Options) {
  const target = options.state.deleteTarget;
  if (!target) return;
  void options.mutation.run({
    key: `delete:${target.log_id}`,
    failureMessage: options.resources.t('messages.deleteFailure'),
    action: () => deleteSystemLog(target.log_id),
    onSuccess: () => {
      options.state.table.onResetCursor();
      options.state.setDeleteTarget(null);
      toast.success(options.resources.t('messages.deleteSuccess'));
    },
  });
}

function confirmBatchDelete(options: Options) {
  void options.mutation.run({
    key: 'delete:batch',
    failureMessage: options.resources.t('messages.deleteFailure'),
    action: () => deleteSystemLogs(options.state.table.selected),
    onSuccess: () => {
      options.state.table.onResetCursor();
      options.state.setBatchOpen(false);
      toast.success(options.resources.t('messages.deleteSuccess'));
    },
  });
}

function requestClean(options: Options) {
  if (options.state.cleanupExecutionId) return;
  const query = applyActionFilter(options);
  if (!query) return;
  const revision = beginCleanupPreview(options.state);
  void options.mutation.run({
    key: 'clean:count',
    failureMessage: options.resources.t('messages.cleanFailure'),
    action: () => countSystemLogsForCleanup(query),
    onSuccess: (result) => {
      if (!acceptsSystemLogCleanupPreview(options.state.cleanPreviewRevision.current, revision))
        return;
      options.state.setCleanPreview(createSystemLogCleanupPreview(query, result.count));
    },
  });
}

function confirmClean(options: Options) {
  const preview = options.state.cleanPreview;
  if (!preview || options.state.cleanupExecutionId) return;
  void options.mutation.run({
    key: 'clean:confirm',
    failureMessage: options.resources.t('messages.cleanFailure'),
    action: () => cleanSystemLogs(preview.query),
    onSuccess: (result) => {
      invalidateCleanupPreview(options.state);
      options.state.trackCleanupExecution(result.execution_id);
      toast.info(
        options.resources.t('messages.cleanAccepted', { executionId: result.execution_id })
      );
    },
  });
}

function submitExport(options: Options) {
  const query = applyActionFilter(options);
  if (!query) return;
  void options.mutation.run({
    key: 'export',
    failureMessage: options.resources.t('messages.exportFailure'),
    action: () => exportSystemLogs(query),
  });
}

function applyActionFilter(options: Options) {
  const resolution = resolveSystemLogActionFilter(options.state.filterDraft);
  if (resolution.kind === 'invalid') {
    options.state.setFilterError(resolution.error);
    toast.error(options.resources.t(FILTER_ERROR_KEYS[resolution.error]));
    return null;
  }
  if (resolution.kind === 'missing_range') {
    options.state.setFilterError(null);
    toast.error(options.resources.t('messages.timeRangeRequired'));
    return null;
  }
  applySystemLogFilters(options.state, options.state.filterDraft);
  return resolution.query;
}

function applySystemLogFilters(state: SystemLogState, draft: SystemLogFilters) {
  const transition = applySystemLogFilterDraft(state.filterQuery, draft);
  state.setFilterDraft(draft);
  state.setFilterError(transition.error);
  invalidateCleanupPreview(state);
  if (!transition.resetTable) return;
  state.setFilterQuery(transition.query);
  state.table.onResetCursor();
}

function filterDraftError(draft: SystemLogFilters): SystemLogFilterError | null {
  const result = toSystemLogQuery(draft);
  return result.ok ? null : result.error;
}

function beginCleanupPreview(state: SystemLogState) {
  state.cleanPreviewRevision.current += 1;
  state.setCleanPreview(null);
  return state.cleanPreviewRevision.current;
}

function invalidateCleanupPreview(state: SystemLogState) {
  state.cleanPreviewRevision.current += 1;
  state.setCleanPreview(null);
}

function resourcesErrorMessage(error: unknown, fallback: string) {
  return error ? apiMutationErrorMessage(error, fallback) : null;
}
