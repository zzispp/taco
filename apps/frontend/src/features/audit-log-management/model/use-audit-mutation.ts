'use client';

import { useRef, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

type MutationOptions<T> = Readonly<{
  key: string;
  failureMessage: string;
  action: () => Promise<T>;
  onSuccess?: (result: T) => void | Promise<void>;
}>;

export function useAuditMutation() {
  const [pending, setPending] = useState<ReadonlySet<string>>(() => new Set());
  const activeKeys = useRef<ReadonlySet<string>>(new Set());
  const run = useCallback(async <T>(options: MutationOptions<T>) => {
    if (activeKeys.current.has(options.key)) return;
    activeKeys.current = new Set(activeKeys.current).add(options.key);
    setPending(activeKeys.current);
    try {
      const result = await options.action();
      await options.onSuccess?.(result);
    } catch (error) {
      toast.error(error instanceof Error ? getErrorMessage(error) : options.failureMessage);
    } finally {
      const next = new Set(activeKeys.current);
      next.delete(options.key);
      activeKeys.current = next;
      setPending(next);
    }
  }, []);
  return { pending, run };
}
