import { useRef, useEffect } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import {
  type SystemLogCleanupExecution,
  SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
  LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
} from 'src/entities/system-log';

type Translate = (key: string, values?: Record<string, number>) => string;

export type CleanupExecutionCopy = Readonly<{
  stateKey: string;
  metricsKey: string;
  values?: Record<string, number>;
}>;

type Options = Readonly<{
  execution: SystemLogCleanupExecution | undefined;
  onTerminal: () => void;
  onClear: () => void;
  t: Translate;
}>;

export function useCleanupExecutionNotification(options: Options) {
  const { execution, onClear, onTerminal, t } = options;
  const handledExecution = useRef<string | null>(null);
  useEffect(() => {
    if (!execution || !isTerminal(execution.state)) return;
    if (handledExecution.current === execution.execution_id) return;
    handledExecution.current = execution.execution_id;
    onTerminal();
    if (execution.state === 'succeeded') {
      toast.success(cleanupExecutionMessage(execution, t));
    } else {
      toast.error(cleanupExecutionMessage(execution, t));
    }
    onClear();
  }, [execution, onClear, onTerminal, t]);
}

export function cleanupExecutionMessage(execution: SystemLogCleanupExecution | undefined, t: Translate) {
  const copy = cleanupExecutionCopy(execution);
  return `${t(copy.stateKey)} ${t(copy.metricsKey, copy.values)}`;
}

export function cleanupExecutionCopy(execution: SystemLogCleanupExecution | undefined): CleanupExecutionCopy {
  if (!execution) {
    return { stateKey: 'cleanupStates.pending', metricsKey: 'cleanupMetrics.unavailable' };
  }
  if (execution.detail_schema_version === SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION) {
    return {
      stateKey: `cleanupStates.${execution.state}`,
      metricsKey: 'cleanupMetrics.current',
      values: currentMetricValues(execution),
    };
  }
  if (execution.detail_schema_version === LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION) {
    return {
      stateKey: `cleanupStates.${execution.state}`,
      metricsKey: 'cleanupMetrics.legacy',
      values: legacyMetricValues(execution),
    };
  }
  return { stateKey: `cleanupStates.${execution.state}`, metricsKey: 'cleanupMetrics.unavailable' };
}

function currentMetricValues(execution: SystemLogCleanupExecution) {
  if (execution.detail_schema_version !== SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION) {
    throw new Error('Expected current system-log cleanup metrics');
  }
  return {
    rowsDeleted: execution.rows_deleted,
    droppedPartitions: execution.dropped_partitions,
    batches: execution.batches,
  };
}

function legacyMetricValues(execution: SystemLogCleanupExecution) {
  if (execution.detail_schema_version !== LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION) {
    throw new Error('Expected legacy system-log cleanup metrics');
  }
  return {
    legacyTotalDeleted: execution.legacy_total_deleted,
    batches: execution.batches,
  };
}

function isTerminal(state: SystemLogCleanupExecution['state']) {
  return state !== 'pending' && state !== 'running';
}
