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
      <DeleteDialog
        open={batchDeleteTypeOpen}
        name={String(selectedTypeCount)}
        t={t}
        onClose={onBatchDeleteTypeClose}
        onDelete={onBatchDeleteTypes}
      />
      <DeleteDialog
        open={batchDeleteDataOpen}
        name={String(selectedDataCount)}
        t={t}
        onClose={onBatchDeleteDataClose}
        onDelete={onBatchDeleteData}
      />
      <DeleteDialog
        open={Boolean(deleteType)}
        name={deleteType?.dict_name ?? ''}
        t={t}
        onClose={onDeleteTypeClose}
        onDelete={onDeleteType}
      />
      <DeleteDialog
        open={Boolean(deleteData)}
        name={deleteData?.dict_label ?? ''}
        t={t}
        onClose={onDeleteDataClose}
        onDelete={onDeleteData}
      />
    </>
  );
}

type DeleteDialogProps = {
  open: boolean;
  name: string;
  t: TranslateFn;
  onClose: () => void;
  onDelete: () => void;
};

function DeleteDialog({ open, name, t, onClose, onDelete }: DeleteDialogProps) {
  return (
    <ConfirmDialog
      open={open}
      onClose={onClose}
      title={t('common.delete')}
      content={t('dialogs.deleteContent', { name })}
      cancelText={t('common.cancel')}
      action={deleteAction(t, onDelete)}
    />
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
