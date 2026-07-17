'use client';

import type { SystemLogCleanupPreview } from './cleanup-preview';
import type {
  SystemLogFilters,
  SystemLogSummary,
  SystemLogFilterError,
  SystemLogFilterQuery,
} from 'src/entities/system-log';

import { useRef, useState, useCallback } from 'react';

import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { toSystemLogQuery, createDefaultSystemLogFilters } from 'src/entities/system-log';

import { loadCleanupExecutionId, storeCleanupExecutionId } from './cleanup-execution-storage';

export function useSystemLogState() {
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'occurred_at',
    defaultOrder: 'desc',
  });
  const [filterDraft, setFilterDraft] = useState(createDefaultSystemLogFilters);
  const [filterQuery, setFilterQuery] = useState<SystemLogFilterQuery>(() =>
    requiredQuery(filterDraft)
  );
  const [filterError, setFilterError] = useState<SystemLogFilterError | null>(null);
  const [detailTarget, setDetailTarget] = useState<SystemLogSummary | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<SystemLogSummary | null>(null);
  const [batchOpen, setBatchOpen] = useState(false);
  const cleanupExecution = useCleanupExecutionState();
  const cleanPreviewRevision = useRef(0);
  const [cleanPreview, setCleanPreview] = useState<SystemLogCleanupPreview | null>(null);
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
    ...cleanupExecution,
    cleanPreviewRevision,
    cleanPreview,
    setCleanPreview,
  };
}

export type SystemLogState = ReturnType<typeof useSystemLogState>;

function useCleanupExecutionState() {
  const [cleanupExecutionId, setCleanupExecutionId] = useState(loadCleanupExecutionId);
  const [cleanupExecutionDialogOpen, setCleanupExecutionDialogOpen] = useState(
    () => loadCleanupExecutionId() !== null
  );
  const trackCleanupExecution = useCallback((executionId: string) => {
    storeCleanupExecutionId(executionId);
    setCleanupExecutionId(executionId);
    setCleanupExecutionDialogOpen(true);
  }, []);
  const clearCleanupExecution = useCallback(() => {
    storeCleanupExecutionId(null);
    setCleanupExecutionId(null);
    setCleanupExecutionDialogOpen(false);
  }, []);
  const showCleanupExecution = useCallback(() => setCleanupExecutionDialogOpen(true), []);
  const dismissCleanupExecution = useCallback(() => setCleanupExecutionDialogOpen(false), []);
  return {
    cleanupExecutionId,
    cleanupExecutionDialogOpen,
    trackCleanupExecution,
    clearCleanupExecution,
    showCleanupExecution,
    dismissCleanupExecution,
  };
}

function requiredQuery(draft: SystemLogFilters) {
  const result = toSystemLogQuery(draft);
  if (!result.ok) throw new Error(`Invalid initial system log filters: ${result.error}`);
  return result.query;
}
