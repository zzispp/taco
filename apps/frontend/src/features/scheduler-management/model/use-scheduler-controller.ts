'use client';

import type { SchedulerJob, ImportableTask } from 'src/entities/scheduler';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import {
  JOB_STATUS,
  useSchedulerJob,
  useSchedulerJobs,
  SCHEDULER_PERMISSION,
  useImportableSchedulerTasks,
} from 'src/entities/scheduler';

import { useMutationRunner } from './use-mutation-runner';
import { runJob, deleteJob, deleteJobs, exportJobs, updateJobStatus } from '../api';

export function useSchedulerController() {
  const state = useSchedulerState();
  const resources = useSchedulerResources(state);
  const mutation = useMutationRunner();
  const actions = {
    ...useJobActions({ state, resources, mutation }),
    ...useJobDeleteActions({ state, resources, mutation }),
  };
  const detail = {
    target: state.detailTarget,
    data: resources.detail.data,
    loading: resources.detail.isLoading,
    error: resources.detail.error,
  };
  return { state, resources, actions, detail, pending: mutation.pending };
}

export type SchedulerController = ReturnType<typeof useSchedulerController>;

function useSchedulerState() {
  const table = useTable({ defaultRowsPerPage: 10 });
  const [importOpen, setImportOpen] = useState(false);
  const [selectedTask, setSelectedTask] = useState<ImportableTask | null>(null);
  const [editing, setEditing] = useState<SchedulerJob | null>(null);
  const [detailTarget, setDetailTarget] = useState<SchedulerJob | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<SchedulerJob | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  return {
    table,
    importOpen,
    setImportOpen,
    selectedTask,
    setSelectedTask,
    editing,
    setEditing,
    detailTarget,
    setDetailTarget,
    deleteTarget,
    setDeleteTarget,
    batchDeleteOpen,
    setBatchDeleteOpen,
  };
}

function useSchedulerResources(state: ReturnType<typeof useSchedulerState>) {
  const { t } = useTranslate('scheduler');
  const canImport = useHasPermission(SCHEDULER_PERMISSION.JOB_IMPORT);
  const canEdit = useHasPermission(SCHEDULER_PERMISSION.JOB_EDIT);
  const canRemove = useHasPermission(SCHEDULER_PERMISSION.JOB_REMOVE);
  const canExport = useHasPermission(SCHEDULER_PERMISSION.JOB_EXPORT);
  const canRun = useHasPermission(SCHEDULER_PERMISSION.JOB_RUN);
  const canStatus = useHasPermission(SCHEDULER_PERMISSION.JOB_CHANGE_STATUS);
  const canQuery = useHasPermission(SCHEDULER_PERMISSION.JOB_QUERY);
  const jobs = useSchedulerJobs(state.table.page, state.table.rowsPerPage);
  const detail = useSchedulerJob({
    jobId: state.detailTarget?.job_id ?? null,
    canQuery,
  });
  const importable = useImportableSchedulerTasks(canImport);
  return {
    t,
    jobs,
    detail,
    importable,
    canImport,
    canEdit,
    canRemove,
    canExport,
    canRun,
    canStatus,
    canViewDetail: canQuery,
  };
}

type ActionOptions = {
  state: ReturnType<typeof useSchedulerState>;
  resources: ReturnType<typeof useSchedulerResources>;
  mutation: ReturnType<typeof useMutationRunner>;
};

function useJobActions(options: ActionOptions) {
  const openDetail = useCallback(
    (job: SchedulerJob) => options.state.setDetailTarget(job),
    [options]
  );
  const closeDetail = useCallback(() => options.state.setDetailTarget(null), [options]);
  const run = useCallback(
    (job: SchedulerJob) =>
      options.mutation.run({
        key: `run:${job.job_id}`,
        failureMessage: options.resources.t('mutation.runFailed'),
        action: () => runJob(job.job_id),
        onSuccess: () => {
          toast.success(options.resources.t('runAccepted'));
        },
      }),
    [options]
  );
  const updateStatus = useCallback(
    (job: SchedulerJob) =>
      options.mutation.run({
        key: `status:${job.job_id}`,
        failureMessage: options.resources.t('mutation.statusFailed'),
        action: () =>
          updateJobStatus(
            job.job_id,
            job.status === JOB_STATUS.NORMAL ? JOB_STATUS.PAUSED : JOB_STATUS.NORMAL,
            { canRefreshImportableTasks: options.resources.canImport }
          ),
      }),
    [options]
  );
  const submitExport = useCallback(
    () =>
      options.mutation.run({
        key: 'export',
        failureMessage: options.resources.t('mutation.exportFailed'),
        action: () => exportJobs(),
      }),
    [options]
  );
  const closeEditor = useCallback(() => {
    options.state.setSelectedTask(null);
    options.state.setEditing(null);
  }, [options]);
  return { openDetail, closeDetail, run, updateStatus, submitExport, closeEditor };
}

function useJobDeleteActions(options: ActionOptions) {
  const confirmDelete = useCallback(() => {
    const job = options.state.deleteTarget;
    if (!job) return Promise.resolve();
    return options.mutation.run({
      key: `delete:${job.job_id}`,
      failureMessage: options.resources.t('mutation.deleteFailed'),
      action: () =>
        deleteJob(job.job_id, {
          canRefreshImportableTasks: options.resources.canImport,
        }),
      onSuccess: () => {
        options.state.setDeleteTarget(null);
        options.state.table.setSelected((current) => current.filter((id) => id !== job.job_id));
      },
    });
  }, [options]);
  const confirmBatchDelete = useCallback(
    () =>
      options.mutation.run({
        key: 'delete:batch',
        failureMessage: options.resources.t('mutation.deleteFailed'),
        action: () =>
          deleteJobs(options.state.table.selected, {
            canRefreshImportableTasks: options.resources.canImport,
          }),
        onSuccess: () => {
          options.state.table.setSelected([]);
          options.state.setBatchDeleteOpen(false);
        },
      }),
    [options]
  );
  return { confirmDelete, confirmBatchDelete };
}
