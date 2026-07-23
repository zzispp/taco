'use client';

import type { FileSpace } from 'src/entities/file';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { usePendingMutation } from 'src/shared/api/use-pending-mutation';

import { updateFileSpace } from '../api';
import { quotaBytesToGib, quotaGibToBytes } from './space-quota';

export function useSpaceQuotaEditor(canEdit: boolean) {
  const { t } = useTranslate('admin');
  const mutation = usePendingMutation();
  const [target, setTarget] = useState<FileSpace | null>(null);
  const [quotaGib, setQuotaGib] = useState('');
  const quotaBytes = quotaGibToBytes(quotaGib);
  const mutationKey = target ? `space-quota:${target.id}` : 'space-quota';
  const pending = mutation.pending.has(mutationKey);

  const open = useCallback(
    (space: FileSpace) => {
      if (!canEdit) return;
      setTarget(space);
      setQuotaGib(quotaBytesToGib(space.quota_bytes));
    },
    [canEdit]
  );
  const close = useCallback(() => {
    setTarget(null);
    setQuotaGib('');
  }, []);
  const update = useCallback(
    (quota: number | null) => {
      if (!target || !canEdit || (quota === null && pending)) return;
      void mutation.run({
        key: mutationKey,
        failureMessage: t('file.messages.quotaUpdateFailed'),
        action: () => updateFileSpace(target.id, { quota_bytes: quota }),
        onSuccess: () => {
          close();
          toast.success(t('file.messages.quotaUpdated'));
        },
      });
    },
    [canEdit, close, mutation, mutationKey, pending, t, target]
  );
  const save = useCallback(() => quotaBytes !== null && update(quotaBytes), [quotaBytes, update]);
  const reset = useCallback(() => update(null), [update]);

  return { target, quotaGib, quotaBytes, pending, open, close, save, reset, setQuotaGib };
}

export type SpaceQuotaEditor = ReturnType<typeof useSpaceQuotaEditor>;
