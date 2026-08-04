export const SYSTEM_LOG_LEVELS = ['trace', 'debug', 'info', 'warn', 'error'] as const;

export type SystemLogLevel = (typeof SYSTEM_LOG_LEVELS)[number];

export type SystemLogSummary = Readonly<{
  log_id: string;
  occurred_at: string;
  level: SystemLogLevel;
  target: string;
  message: string;
}>;

export type SystemLogDetail = SystemLogSummary &
  Readonly<{
    fields: Record<string, unknown>;
  }>;

export type SystemLogFilters = Readonly<{
  keyword: string;
  levels: readonly SystemLogLevel[];
  target: string;
  begin_time: string;
  end_time: string;
}>;

export type SystemLogFilterQuery = Readonly<{
  keyword?: string;
  levels?: string;
  target?: string;
  begin_time?: string;
  end_time?: string;
}>;

export type SystemLogCleanupCount = Readonly<{ count: number }>;

export type SystemLogCleanupAccepted = Readonly<{
  accepted: boolean;
  execution_id: string;
}>;

export const SYSTEM_LOG_CLEANUP_EXECUTION_STATES = [
  'pending',
  'running',
  'succeeded',
  'failed',
  'skipped',
  'interrupted',
] as const;

export type SystemLogCleanupExecutionState = (typeof SYSTEM_LOG_CLEANUP_EXECUTION_STATES)[number];

export const LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION = 1;
export const SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION = 2;

type SystemLogCleanupExecutionBase = Readonly<{
  execution_id: string;
  state: SystemLogCleanupExecutionState;
}>;

type CurrentSystemLogCleanupExecution = SystemLogCleanupExecutionBase &
  Readonly<{
    detail_schema_version: typeof SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION;
    rows_deleted: number;
    dropped_partitions: number;
    legacy_total_deleted: null;
    batches: number;
  }>;

type LegacySystemLogCleanupExecution = SystemLogCleanupExecutionBase &
  Readonly<{
    detail_schema_version: typeof LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION;
    rows_deleted: null;
    dropped_partitions: null;
    legacy_total_deleted: number;
    batches: number;
  }>;

type UnreportedSystemLogCleanupExecution = SystemLogCleanupExecutionBase &
  Readonly<{
    detail_schema_version: null;
    rows_deleted: null;
    dropped_partitions: null;
    legacy_total_deleted: null;
    batches: null;
  }>;

export type SystemLogCleanupExecution =
  | CurrentSystemLogCleanupExecution
  | LegacySystemLogCleanupExecution
  | UnreportedSystemLogCleanupExecution;
