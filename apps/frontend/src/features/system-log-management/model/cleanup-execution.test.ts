import type { SystemLogCleanupExecution } from 'src/entities/system-log';

import { it, expect, describe } from 'vitest';

import {
  SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
  LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
} from 'src/entities/system-log';

import { cleanupExecutionCopy } from './cleanup-execution';

describe('system log cleanup execution copy', () => {
  it('keeps current row and partition metrics distinct', () => {
    const copy = cleanupExecutionCopy(currentExecution());

    expect(copy).toEqual({
      stateKey: 'cleanupStates.succeeded',
      metricsKey: 'cleanupMetrics.current',
      values: { rowsDeleted: 5, droppedPartitions: 2, batches: 3 },
    });
  });

  it('keeps the legacy total out of current metric fields', () => {
    const copy = cleanupExecutionCopy(legacyExecution());

    expect(copy).toEqual({
      stateKey: 'cleanupStates.failed',
      metricsKey: 'cleanupMetrics.legacy',
      values: { legacyTotalDeleted: 7, batches: 4 },
    });
  });

  it('does not invent zero metrics when an execution has no detail', () => {
    const copy = cleanupExecutionCopy({
      execution_id: 'execution',
      state: 'interrupted',
      detail_schema_version: null,
      rows_deleted: null,
      dropped_partitions: null,
      legacy_total_deleted: null,
      batches: null,
    });

    expect(copy).toEqual({
      stateKey: 'cleanupStates.interrupted',
      metricsKey: 'cleanupMetrics.unavailable',
    });
  });
});

function currentExecution(): SystemLogCleanupExecution {
  return {
    execution_id: 'execution',
    state: 'succeeded',
    detail_schema_version: SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
    rows_deleted: 5,
    dropped_partitions: 2,
    legacy_total_deleted: null,
    batches: 3,
  };
}

function legacyExecution(): SystemLogCleanupExecution {
  return {
    execution_id: 'execution',
    state: 'failed',
    detail_schema_version: LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION,
    rows_deleted: null,
    dropped_partitions: null,
    legacy_total_deleted: 7,
    batches: 4,
  };
}
