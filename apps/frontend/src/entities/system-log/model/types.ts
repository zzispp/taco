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

export type SystemLogCleanupExecution = Readonly<{
  execution_id: string;
  state: SystemLogCleanupExecutionState;
  deleted: number | null;
  batches: number | null;
}>;
