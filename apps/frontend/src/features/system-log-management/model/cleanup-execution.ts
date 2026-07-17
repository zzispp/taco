import type { SystemLogCleanupExecution } from 'src/entities/system-log';

import { useRef, useEffect } from 'react';

import { toast } from 'src/shared/ui/snackbar';

type Translate = (key: string, values?: Record<string, number>) => string;

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
      toast.success(t('messages.cleanCompleted', completedParams(execution)));
    } else {
      toast.error(t('messages.cleanExecutionFailed', completedParams(execution)));
    }
    onClear();
  }, [execution, onClear, onTerminal, t]);
}

function completedParams(execution: SystemLogCleanupExecution) {
  return {
    count: execution.deleted ?? 0,
    batches: execution.batches ?? 0,
  };
}

function isTerminal(state: SystemLogCleanupExecution['state']) {
  return state !== 'pending' && state !== 'running';
}
