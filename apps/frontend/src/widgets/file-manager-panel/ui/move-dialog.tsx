'use client';

import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import { useTheme } from '@mui/material/styles';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import useMediaQuery from '@mui/material/useMediaQuery';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Iconify } from 'src/shared/ui/iconify';
import { CursorPagination } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { FileThumbnail } from 'src/shared/ui/file-thumbnail';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { MoveFolderDialog } from './move-folder-dialog';
import { MoveDestinationBrowser } from './move-destination-browser';

const MOVE_DIALOG_BODY_HEIGHT = 360;

export function MoveDialog({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const theme = useTheme();
  const fullScreen = useMediaQuery(theme.breakpoints.down('sm'));
  const entry = controller.state.moveTarget;
  if (!entry) return null;
  const busy = controller.pending.has(`move:${entry.id}`);
  const close = () => {
    if (!busy) controller.actions.closeMove();
  };
  return (
    <>
      <Dialog fullWidth fullScreen={fullScreen} maxWidth="md" open onClose={close}>
        <DialogTitle>{t('file.moveTitle')}</DialogTitle>
        <DialogContent dividers>
          <MoveSource entry={entry} />
          <MoveDialogBody controller={controller} busy={busy} />
        </DialogContent>
        <DialogActions sx={{ flexWrap: 'wrap' }}>
          {controller.resources.moveBrowser && !controller.resources.canSubmitMove ? (
            <Typography variant="caption" color="text.secondary" sx={{ mr: 'auto' }}>
              {t('file.moveSameLocation')}
            </Typography>
          ) : null}
          <Button onClick={close} disabled={busy}>
            {t('file.actions.cancel')}
          </Button>
          <Button
            variant="contained"
            loading={busy}
            startIcon={<Iconify icon="eva:arrow-forward-fill" />}
            disabled={!controller.resources.canSubmitMove || !controller.resources.moveBrowser}
            onClick={controller.actions.moveEntry}
          >
            {t('file.actions.moveHere')}
          </Button>
        </DialogActions>
      </Dialog>
      <MoveFolderDialog controller={controller} />
    </>
  );
}

function MoveSource({ entry }: { entry: FileEntry }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" spacing={1.5} alignItems="center" sx={{ mb: 2 }}>
      <FileThumbnail
        file={entry.type === 'folder' ? 'folder' : entry.name}
        sx={{ width: 44, height: 44, flexShrink: 0 }}
      />
      <Box sx={{ minWidth: 0 }}>
        <Typography variant="caption" color="text.secondary">
          {t('file.moveSource')}
        </Typography>
        <Typography variant="subtitle2" noWrap title={entry.name}>
          {entry.name}
        </Typography>
      </Box>
    </Stack>
  );
}

function MoveDialogBody({
  controller,
  busy,
}: Readonly<{ controller: FileManagerController; busy: boolean }>) {
  const { t } = useTranslate('admin');
  const folders = controller.resources.folders;
  const loading = folders.isLoading || controller.resources.moveTrailLoading;
  const error = folders.error || controller.resources.moveTrailError;
  if (loading) {
    return <LoadingScreen portal={false} sx={{ minHeight: MOVE_DIALOG_BODY_HEIGHT }} />;
  }
  if (error || !controller.resources.moveBrowser) {
    return (
      <Alert severity="error">
        {getErrorMessage(error) || t('file.messages.folderListFailed')}
      </Alert>
    );
  }
  return (
    <Stack spacing={1}>
      <MoveDestinationBrowser
        browser={controller.resources.moveBrowser}
        canMoveHere={controller.resources.canSubmitMove}
        canCreateFolder={controller.permissions.canAddFolder}
        disabled={busy}
        onNavigate={controller.state.setMoveDestinationId}
        onCreateFolder={controller.actions.openMoveFolderDialog}
        onMoveHere={controller.actions.moveEntry}
      />
      <MoveFolderPagination controller={controller} />
    </Stack>
  );
}

function MoveFolderPagination({ controller }: { controller: FileManagerController }) {
  const { folders, moveFolderTable: table } = controller.resources;
  return (
    <CursorPagination
      limit={table.limit}
      itemCount={folders.itemCount}
      visitedBatchIndex={table.visitedBatchIndex}
      hasPrevious={folders.hasPrevious}
      hasNext={folders.hasNext}
      pending={folders.isValidating}
      onPrevious={() => table.onPreviousCursor(folders.previousCursor)}
      onNext={() => table.onNextCursor(folders.nextCursor)}
      onLimitChange={table.onChangeLimit}
    />
  );
}
