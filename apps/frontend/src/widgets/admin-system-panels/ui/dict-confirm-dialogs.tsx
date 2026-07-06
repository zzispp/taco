import type { TranslateFn } from 'src/shared/i18n';
import type { DictData, DictType } from 'src/entities/system';

import Button from '@mui/material/Button';

import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

export function DictConfirmDialogs({
  t,
  deleteType,
  deleteData,
  batchDeleteTypeOpen,
  batchDeleteDataOpen,
  selectedTypeCount,
  selectedDataCount,
  onBatchDeleteTypeClose,
  onBatchDeleteDataClose,
  onDeleteTypeClose,
  onDeleteDataClose,
  onBatchDeleteTypes,
  onBatchDeleteData,
  onDeleteType,
  onDeleteData,
}: DictConfirmDialogsProps) {
  return (
    <>
      <ConfirmDialog
        open={batchDeleteTypeOpen}
        onClose={onBatchDeleteTypeClose}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: String(selectedTypeCount) })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onBatchDeleteTypes)}
      />
      <ConfirmDialog
        open={batchDeleteDataOpen}
        onClose={onBatchDeleteDataClose}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: String(selectedDataCount) })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onBatchDeleteData)}
      />
      <ConfirmDialog
        open={!!deleteType}
        onClose={onDeleteTypeClose}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: deleteType?.dict_name ?? '' })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onDeleteType)}
      />
      <ConfirmDialog
        open={!!deleteData}
        onClose={onDeleteDataClose}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: deleteData?.dict_label ?? '' })}
        cancelText={t('common.cancel')}
        action={deleteAction(t, onDeleteData)}
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

type DictConfirmDialogsProps = {
  t: TranslateFn;
  deleteType: DictType | null;
  deleteData: DictData | null;
  batchDeleteTypeOpen: boolean;
  batchDeleteDataOpen: boolean;
  selectedTypeCount: number;
  selectedDataCount: number;
  onBatchDeleteTypeClose: () => void;
  onBatchDeleteDataClose: () => void;
  onDeleteTypeClose: () => void;
  onDeleteDataClose: () => void;
  onBatchDeleteTypes: () => void;
  onBatchDeleteData: () => void;
  onDeleteType: () => void;
  onDeleteData: () => void;
};
