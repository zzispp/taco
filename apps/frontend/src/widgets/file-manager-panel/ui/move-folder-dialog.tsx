'use client';

import type { FileManagerController } from 'src/features/file-management';

import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/shared/i18n/use-locales';

const MOVE_FOLDER_MUTATION_KEY = 'move-folder:create';

export function MoveFolderDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const open = controller.state.moveFolderOpen;
  const pending = controller.pending.has(MOVE_FOLDER_MUTATION_KEY);
  const close = () => {
    if (!pending) controller.actions.closeMoveFolderDialog();
  };
  return (
    <Dialog fullWidth maxWidth="xs" open={open} onClose={close}>
      <DialogTitle>{t('file.newFolderTitle')}</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          fullWidth
          label={t('file.folderName')}
          value={controller.state.moveFolderName}
          onChange={(event) => controller.state.setMoveFolderName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && !pending) controller.actions.submitMoveFolder();
          }}
          sx={{ mt: 1 }}
        />
      </DialogContent>
      <DialogActions>
        <Button disabled={pending} onClick={close}>
          {t('file.actions.cancel')}
        </Button>
        <Button
          variant="contained"
          disabled={pending || !controller.state.moveFolderName.trim()}
          onClick={controller.actions.submitMoveFolder}
        >
          {t('file.actions.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
