'use client';

import type { SchedulerJobLog, SchedulerJobLogQuery } from 'src/entities/scheduler';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { useHasPermission } from 'src/entities/session';
import {
  useSchedulerJobLogs,
  SCHEDULER_PERMISSION,
  useSchedulerJobLogDetail,
} from 'src/entities/scheduler';

import { copyTextWithFeedback } from './clipboard';
import { useMutationRunner } from './use-mutation-runner';
import { deleteJobLog, clearJobLogs, deleteJobLogs, exportJobLogs } from '../api';
import {
  requireSchedulerJobLogDeleteTarget,
  requireUsableSchedulerJobLogFilters,
} from './mutation-preconditions';
import {
  JOB_LOG_FILTER_ERROR,
  DEFAULT_JOB_LOG_QUERY,
  updateJobLogFilterState,
  isSchedulerJobLogQueryUsable,
  DEFAULT_JOB_LOG_FILTER_DRAFT,
} from './job-log-filters';

const FILTER_ERROR_KEYS = {
  [JOB_LOG_FILTER_ERROR.INVALID_DATE_TIME]: 'filters.invalidDateTime',
  [JOB_LOG_FILTER_ERROR.INVALID_RANGE]: 'filters.invalidRange',
} as const;

export function useJobLogController() {
  const state = useJobLogState();
  const resources = useJobLogResources(state);
  const mutation = useMutationRunner();
  const actions = useJobLogActions({ state, resources, mutation });
  const detail = {
    target: state.detailTarget,
    data: resources.detail.data,
    loading: resources.detail.isLoading,
    error: resources.detail.error,
  };
  return { state, resources, actions, detail, pending: mutation.pending };
}

export type JobLogController = ReturnType<typeof useJobLogController>;

function useJobLogState() {
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const [filterDraft, setFilterDraft] = useState(DEFAULT_JOB_LOG_FILTER_DRAFT);
  const [query, setQuery] = useState<SchedulerJobLogQuery>(DEFAULT_JOB_LOG_QUERY);
  const [filterError, setFilterError] = useState<keyof typeof FILTER_ERROR_KEYS | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<SchedulerJobLog | null>(null);
  const [batchOpen, setBatchOpen] = useState(false);
  const [cleanOpen, setCleanOpen] = useState(false);
  const [detailTarget, setDetailTarget] = useState<SchedulerJobLog | null>(null);
  return {
    table,
    query,
    setQuery,
    filterDraft,
    setFilterDraft,
    filterError,
    setFilterError,
    deleteTarget,
    setDeleteTarget,
    batchOpen,
    setBatchOpen,
    cleanOpen,
    setCleanOpen,
    detailTarget,
    setDetailTarget,
  };
}

function useJobLogResources(state: ReturnType<typeof useJobLogState>) {
  const { t } = useTranslate('scheduler');
  const canRemove = useHasPermission(SCHEDULER_PERMISSION.JOB_LOG_REMOVE);
  const canExport = useHasPermission(SCHEDULER_PERMISSION.JOB_LOG_EXPORT);
  const canQuery = useHasPermission(SCHEDULER_PERMISSION.JOB_LOG_QUERY);
  const canDetail = useHasPermission(SCHEDULER_PERMISSION.JOB_LOG_DETAIL);
  const logs = useSchedulerJobLogs(state.table.cursorRequest, state.query);
  const detail = useSchedulerJobLogDetail({
    executionId: state.detailTarget?.execution_id ?? null,
    canQuery,
    canDetail,
  });
  return {
    t,
    logs,
    detail,
    canRemove,
    canExport,
    filtersValid: isSchedulerJobLogQueryUsable(state.filterError),
    filterErrorMessage: state.filterError ? t(FILTER_ERROR_KEYS[state.filterError]) : null,
    canViewDetail: canQuery && canDetail,
  };
}

type LogActionOptions = {
  state: ReturnType<typeof useJobLogState>;
  resources: ReturnType<typeof useJobLogResources>;
  mutation: ReturnType<typeof useMutationRunner>;
};

function useJobLogActions(options: LogActionOptions) {
  const changeFilters = useJobLogFilterAction(options.state);
  const resetFilters = useCallback(
    () => changeFilters(DEFAULT_JOB_LOG_FILTER_DRAFT),
    [changeFilters]
  );
  const viewActions = useJobLogViewActions(options.state);
  const deleteActions = useJobLogDeleteActions(options);
  const maintenanceActions = useJobLogMaintenanceActions(options);
  const copyExecutionId = useCopyExecutionIdAction(options.resources);
  return {
    ...viewActions,
    ...deleteActions,
    ...maintenanceActions,
    changeFilters,
    resetFilters,
    copyExecutionId,
  };
}

function useJobLogViewActions(state: LogActionOptions['state']) {
  const openDetail = useCallback((log: SchedulerJobLog) => state.setDetailTarget(log), [state]);
  const closeDetail = useCallback(() => state.setDetailTarget(null), [state]);
  return { openDetail, closeDetail };
}

function useJobLogDeleteActions(options: LogActionOptions) {
  const confirmDelete = useCallback(() => {
    const log = requireSchedulerJobLogDeleteTarget(options.state.deleteTarget);
    return options.mutation.run({
      key: `delete:${log.execution_id}`,
      failureMessage: options.resources.t('mutation.deleteFailed'),
      action: () => deleteJobLog(log.execution_id),
      onSuccess: () => {
        options.state.table.onResetCursor();
        options.state.setDeleteTarget(null);
      },
    });
  }, [options]);
  const confirmBatchDelete = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:batch',
        failureMessage: options.resources.t('mutation.deleteFailed'),
        action: () => deleteJobLogs(options.state.table.selected),
        onSuccess: () => {
          options.state.table.onResetCursor();
          options.state.setBatchOpen(false);
        },
      }),
    [options]
  );
  return { confirmDelete, confirmBatchDelete };
}

function useJobLogMaintenanceActions(options: LogActionOptions) {
  const confirmClean = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:clean',
        failureMessage: options.resources.t('mutation.deleteFailed'),
        action: clearJobLogs,
        onSuccess: () => {
          options.state.table.onResetCursor();
          options.state.setCleanOpen(false);
        },
      }),
    [options]
  );
  const submitExport = useCallback(() => {
    requireUsableSchedulerJobLogFilters(options.resources.filtersValid);
    return options.mutation.run({
      key: 'export',
      failureMessage: options.resources.t('mutation.exportFailed'),
      action: () => exportJobLogs(options.state.query),
    });
  }, [options]);
  return { confirmClean, submitExport };
}

function useJobLogFilterAction(state: ReturnType<typeof useJobLogState>) {
  return useCallback(
    (nextDraft: typeof state.filterDraft) => {
      const next = updateJobLogFilterState(state.query, nextDraft);
      state.setFilterDraft(next.draft);
      state.setFilterError(next.error);
      state.setQuery(next.query);
      if (next.resetTable) {
        state.table.onResetCursor();
      }
    },
    [state]
  );
}

function useCopyExecutionIdAction(resources: ReturnType<typeof useJobLogResources>) {
  return useCallback(
    (executionId: string) =>
      copyTextWithFeedback(executionId, {
        success: () => toast.success(resources.t('copyExecutionIdSuccess')),
        failure: () => toast.error(resources.t('copyExecutionIdFailed')),
      }),
    [resources]
  );
}
