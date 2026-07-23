'use client';

import type { FileManagerController } from 'src/features/file-management';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { isUploadQueueBusy, canSubmitUploadQueue } from 'src/features/file-management';

import { MoveDialog } from './move-dialog';
import { UploadQueue } from './upload-queue';

export function FileManagerDialogs({ controller }: { controller: FileManagerController }) {
  return (
    <>
      <UploadDialog controller={controller} />
      <FolderDialog controller={controller} />
      <MoveDialog controller={controller} />
      <BatchActionDialog controller={controller} />
      <DeleteDialog controller={controller} />
    </>
  );
}

function UploadDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const open = controller.state.uploadOpen;
  const items = controller.state.uploadItems;
  const uploading = isUploadQueueBusy(items);
  const canSubmit = canSubmitUploadQueue(items);
  const close = () => {
    if (uploading) return;
    controller.actions.closeUpload();
  };
  const cancel = () => {
    if (uploading) {
      controller.actions.cancelUpload();
      return;
    }
    close();
  };
  return (
    <Dialog fullWidth maxWidth="sm" open={open} onClose={close}>
      <DialogTitle>{t('file.actions.upload')}</DialogTitle>
      <DialogContent>
        <UploadQueue
          items={items}
          disabled={uploading}
          onAppend={controller.state.appendUploadFiles}
          onRemove={controller.state.removeUploadItem}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={cancel}>
          {uploading ? t('file.actions.cancelUpload') : t('file.actions.cancel')}
        </Button>
        <Button
          variant="contained"
          disabled={!canSubmit || !controller.resources.spaceId || uploading}
          startIcon={<Iconify icon="eva:cloud-upload-fill" />}
          onClick={controller.actions.submitUpload}
        >
          {t('file.actions.upload')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function FolderDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  return (
    <Dialog
      fullWidth
      maxWidth="xs"
      open={controller.state.folderOpen}
      onClose={controller.actions.closeFolderDialog}
    >
      <DialogTitle>{t('file.newFolderTitle')}</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          fullWidth
          label={t('file.folderName')}
          value={controller.state.folderName}
          onChange={(event) => controller.state.setFolderName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') controller.actions.submitFolder();
          }}
          sx={{ mt: 1 }}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={controller.actions.closeFolderDialog}>{t('file.actions.cancel')}</Button>
        <Button
          variant="contained"
          disabled={!controller.state.folderName.trim()}
          onClick={controller.actions.submitFolder}
        >
          {t('file.actions.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DeleteDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const entry = controller.state.deleteTarget;
  if (!entry) return null;
  const inTrash = controller.state.mode === 'trash';
  return (
    <ConfirmDialog
      open
      title={inTrash ? t('file.actions.purge') : t('file.actions.delete')}
      content={<Box component="span">{entry.name}</Box>}
      onClose={controller.actions.closeDelete}
      cancelText={t('file.actions.cancel')}
      action={
        <Button color="error" variant="contained" onClick={controller.actions.deleteEntry}>
          {inTrash ? t('file.actions.purge') : t('file.actions.delete')}
        </Button>
      }
    />
  );
}

function BatchActionDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const action = controller.state.batchAction;
  if (!action) return null;
  const count = controller.state.table.selected.length;
  const busy = controller.pending.has(`batch:${action}`);
  const destructive = action !== 'restore';
  return (
    <ConfirmDialog
      open
      title={t(`file.actions.${action}Selected`)}
      content={t('file.batchConfirm', { count })}
      onClose={controller.actions.closeBatchAction}
      cancelText={t('file.actions.cancel')}
      action={
        <Button
          color={destructive ? 'error' : 'primary'}
          variant="contained"
          disabled={busy}
          onClick={controller.actions.executeBatchAction}
        >
          {t(`file.actions.${action}Selected`)}
        </Button>
      }
    />
  );
}
