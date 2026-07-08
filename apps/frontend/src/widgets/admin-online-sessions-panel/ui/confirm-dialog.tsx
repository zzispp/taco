import type { OnlineSession } from 'src/entities/online-session';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

export function ForceLogoutDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: OnlineSession | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('onlineSessions.forceLogout')}
      content={t('onlineSessions.forceLogoutConfirm', { name: target?.userName ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('onlineSessions.forceLogout')}
        </Button>
      }
    />
  );
}
