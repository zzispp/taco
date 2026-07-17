'use client';

import { useRef, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import { apiMutationErrorMessage } from './mutation-error';

type PendingMutationOptions<T> = Readonly<{
  key: string;
  failureMessage: string;
  action: () => Promise<T>;
  onSuccess?: (result: T) => void | Promise<void>;
}>;

export function usePendingMutation() {
  const [pending, setPending] = useState<ReadonlySet<string>>(() => new Set());
  const activeKeys = useRef<ReadonlySet<string>>(new Set());
  const run = useCallback(async <T>(options: PendingMutationOptions<T>) => {
    if (activeKeys.current.has(options.key)) return;
    activeKeys.current = new Set(activeKeys.current).add(options.key);
    setPending(activeKeys.current);
    try {
      const result = await options.action();
      await options.onSuccess?.(result);
    } catch (error) {
      toast.error(apiMutationErrorMessage(error, options.failureMessage));
    } finally {
      const next = new Set(activeKeys.current);
      next.delete(options.key);
      activeKeys.current = next;
      setPending(next);
    }
  }, []);
  return { pending, run };
}
