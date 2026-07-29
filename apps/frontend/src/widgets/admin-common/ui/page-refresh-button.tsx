'use client';

import { useRef, useState } from 'react';

import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { apiMutationErrorMessage } from 'src/shared/api/mutation-error';

type PageRefreshButtonProps = Readonly<{
  onRefresh: () => Promise<void> | void;
  loading?: boolean;
}>;

export function PageRefreshButton({ onRefresh, loading = false }: PageRefreshButtonProps) {
  const { t } = useTranslate('admin');
  const [pending, setPending] = useState(false);
  const inFlight = useRef(false);
  const busy = pending || loading;
  const handleRefresh = async () => {
    if (inFlight.current || loading) return;
    inFlight.current = true;
    try {
      await runPageRefresh({
        onRefresh,
        onPendingChange: setPending,
        onError: (error) =>
          toast.error(apiMutationErrorMessage(error, t('messages.refreshFailed'))),
      });
    } finally {
      inFlight.current = false;
    }
  };

  return (
    <Button
      variant="outlined"
      color="inherit"
      loading={busy}
      disabled={busy}
      startIcon={<Iconify icon="solar:restart-bold" />}
      onClick={() => void handleRefresh()}
    >
      {t('actions.refresh')}
    </Button>
  );
}

type RunPageRefreshOptions = Readonly<{
  onRefresh: () => Promise<void> | void;
  onPendingChange: (pending: boolean) => void;
  onError: (error: unknown) => void;
}>;

export async function runPageRefresh(options: RunPageRefreshOptions) {
  options.onPendingChange(true);
  try {
    await options.onRefresh();
  } catch (error) {
    options.onError(error);
  } finally {
    options.onPendingChange(false);
  }
}
