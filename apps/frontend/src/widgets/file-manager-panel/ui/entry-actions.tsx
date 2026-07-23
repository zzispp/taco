import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  canEditFileEntry,
  canDeleteFileEntry,
  canRestoreFileEntry,
  canViewFileEntryDetails,
} from 'src/features/file-management';

type EntryActionProps = Readonly<{
  entry: FileEntry;
  controller: FileManagerController;
}>;

export function EntryActions({ entry, controller }: EntryActionProps) {
  return (
    <Stack direction="row" spacing={0.25} onDoubleClick={(event) => event.stopPropagation()}>
      <RestoreAction entry={entry} controller={controller} />
      <MoveAction entry={entry} controller={controller} />
      <DetailsAction entry={entry} controller={controller} />
      <DeleteAction entry={entry} controller={controller} />
    </Stack>
  );
}

function RestoreAction({ entry, controller }: EntryActionProps) {
  const { t } = useTranslate('admin');
  if (!canRestoreFileEntry(controller.state.mode, controller.permissions)) return null;
  return (
    <Tooltip title={t('file.actions.restore')}>
      <IconButton size="small" onClick={() => controller.actions.restoreEntry(entry)}>
        <Iconify icon="solar:restart-bold" />
      </IconButton>
    </Tooltip>
  );
}

function MoveAction({ entry, controller }: EntryActionProps) {
  const { t } = useTranslate('admin');
  if (!canEditFileEntry(controller.state.mode, controller.permissions)) return null;
  return (
    <Tooltip title={t('file.actions.move')}>
      <IconButton size="small" onClick={() => controller.actions.requestMove(entry)}>
        <Iconify icon="eva:arrow-forward-fill" />
      </IconButton>
    </Tooltip>
  );
}

function DetailsAction({ entry, controller }: EntryActionProps) {
  const { t } = useTranslate('admin');
  if (!canViewFileEntryDetails(controller.permissions)) return null;
  return (
    <Tooltip title={t('file.details')}>
      <IconButton size="small" onClick={() => controller.actions.openDetail(entry)}>
        <Iconify icon="solar:info-circle-bold" />
      </IconButton>
    </Tooltip>
  );
}

function DeleteAction({ entry, controller }: EntryActionProps) {
  const { t } = useTranslate('admin');
  if (!canDeleteFileEntry(controller.state.mode, controller.permissions)) return null;
  const label = controller.state.mode === 'trash' ? 'file.actions.purge' : 'file.actions.delete';
  return (
    <Tooltip title={t(label)}>
      <IconButton
        size="small"
        color="error"
        onClick={() => controller.actions.requestDelete(entry)}
      >
        <Iconify icon="solar:trash-bin-trash-bold" />
      </IconButton>
    </Tooltip>
  );
}
