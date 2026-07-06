import type { Dept } from 'src/entities/system';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

export function DeptConfirmDialog({
  target,
  onClose,
  onConfirm,
}: {
  target: Dept | null;
  onClose: () => void;
  onConfirm: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <ConfirmDialog
      open={!!target}
      onClose={onClose}
      title={t('common.delete')}
      content={t('dialogs.deleteContent', { name: target?.dept_name ?? '' })}
      cancelText={t('common.cancel')}
      action={
        <Button variant="contained" color="error" onClick={onConfirm}>
          {t('common.delete')}
        </Button>
      }
    />
  );
}
