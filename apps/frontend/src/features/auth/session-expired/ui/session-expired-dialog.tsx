'use client';

import { useState, useEffect, useCallback } from 'react';

import Button from '@mui/material/Button';

import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { subscribeAuthSessionRejected } from 'src/shared/api/http-client';

import { setSession } from 'src/entities/session';

export function SessionExpiredDialog() {
  const { t } = useTranslate('admin');
  const router = useRouter();
  const [open, setOpen] = useState(false);

  useEffect(() => subscribeAuthSessionRejected(() => setOpen(true)), []);

  const stayOnPage = useCallback(() => setOpen(false), []);
  const relogin = useCallback(async () => {
    await setSession(null);
    setOpen(false);
    router.replace(paths.auth.jwt.signIn);
  }, [router]);

  return (
    <ConfirmDialog
      open={open}
      title={t('authExpired.title')}
      content={t('authExpired.content')}
      onClose={stayOnPage}
      cancelText={t('authExpired.stay')}
      action={
        <Button variant="contained" color="primary" onClick={relogin}>
          {t('authExpired.relogin')}
        </Button>
      }
    />
  );
}
