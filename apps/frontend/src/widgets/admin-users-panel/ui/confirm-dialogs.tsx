import type { TranslateFn } from 'src/shared/i18n';
import type { SystemUser } from 'src/entities/user';

import Button from '@mui/material/Button';

import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

export function UserConfirmDialogs({
  t,
  deleteTarget,
  batchDeleteOpen,
  selectedCount,
  onBatchDeleteClose,
  onDeleteClose,
  onBatchDeleteConfirm,
  onDeleteConfirm,
}: {
  t: TranslateFn;
  deleteTarget: SystemUser | null;
  batchDeleteOpen: boolean;
  selectedCount: number;
  onBatchDeleteClose: () => void;
  onDeleteClose: () => void;
  onBatchDeleteConfirm: () => void;
  onDeleteConfirm: () => void;
}) {
  return (
    <>
      <ConfirmDialog
        open={batchDeleteOpen}
        onClose={onBatchDeleteClose}
        title={t('dialogs.deleteUser')}
        content={t('dialogs.deleteContent', { name: String(selectedCount) })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onBatchDeleteConfirm)}
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={onDeleteClose}
        title={t('dialogs.deleteUser')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.username ?? '' })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onDeleteConfirm)}
      />
    </>
  );
}

function deleteAction(t: TranslateFn, onClick: () => void) {
  return (
    <Button variant="contained" color="error" onClick={onClick}>
      {t('common.delete')}
    </Button>
  );
}
