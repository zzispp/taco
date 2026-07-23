import type { MouseEvent, KeyboardEvent, MouseEventHandler } from 'react';
import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import Typography from '@mui/material/Typography';

import { fData } from 'src/shared/lib/format-number';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';

import { ManagedFileThumbnail } from 'src/entities/file';

import { EntryActions } from './entry-actions';
import { activateFileEntry, openFileEntryDetails } from './entry-interactions';

type EntryViewProps = Readonly<{
  entry: FileEntry;
  controller: FileManagerController;
}>;

type FileEntryContextMenuHandler = (event: MouseEvent<HTMLElement>, entry: FileEntry) => void;

export function FileEntryRow({
  entry,
  controller,
  selected,
  selectable,
  onContextMenu,
}: {
  entry: FileEntry;
  controller: FileManagerController;
  selected: boolean;
  selectable: boolean;
  onContextMenu: FileEntryContextMenuHandler;
}) {
  return (
    <FileEntryTableRow
      entry={entry}
      controller={controller}
      selected={selected}
      selectable={selectable}
      onContextMenu={(event) => onContextMenu(event, entry)}
    />
  );
}

function FileEntryTableRow({
  entry,
  controller,
  selected,
  selectable,
  onContextMenu,
}: EntryViewProps & {
  selected: boolean;
  selectable: boolean;
  onContextMenu: MouseEventHandler<HTMLTableRowElement>;
}) {
  const { t } = useTranslate('admin');
  const folder = entry.type === 'folder';
  const type = folder ? t('file.folder') : (entry.mime_type ?? t('file.file'));
  return (
    <TableRow
      hover
      selected={selected}
      onContextMenu={onContextMenu}
      onDoubleClick={() => activateFileEntry(entry, controller)}
    >
      <EntrySelectionCell
        entry={entry}
        controller={controller}
        selected={selected}
        selectable={selectable}
      />
      <EntryNameCell entry={entry} controller={controller} type={type} />
      <TableCell>{type}</TableCell>
      <TableCell>{folder ? '-' : fData(entry.size_bytes)}</TableCell>
      <TableCell>{fAdminDateTime(entry.updated_at)}</TableCell>
      <TableCell align="right">
        <EntryActions entry={entry} controller={controller} />
      </TableCell>
    </TableRow>
  );
}

function EntrySelectionCell({
  entry,
  controller,
  selected,
  selectable,
}: EntryViewProps & { selected: boolean; selectable: boolean }) {
  if (!selectable) return null;
  return (
    <TableCell padding="checkbox">
      <Checkbox
        checked={selected}
        onChange={() => controller.state.table.onSelectRow(entry.id)}
        onDoubleClick={(event) => event.stopPropagation()}
      />
    </TableCell>
  );
}

function EntryNameCell({ entry, controller, type }: EntryViewProps & { type: string }) {
  const { t } = useTranslate('admin');
  return (
    <TableCell>
      <Stack direction="row" spacing={1.5} alignItems="center" sx={{ minWidth: 260 }}>
        <ManagedFileThumbnail
          entry={entry}
          enabled={onlineThumbnailEnabled(controller)}
          errorLabel={t('file.messages.thumbnailFailed')}
          sx={{ width: 44, height: 44, flexShrink: 0 }}
        />
        <Box sx={{ minWidth: 0 }}>
          <Typography noWrap variant="subtitle2">
            {entry.name}
          </Typography>
          <Typography noWrap variant="caption" color="text.secondary">
            {type}
          </Typography>
        </Box>
      </Stack>
    </TableCell>
  );
}

export function FileEntryCard({
  entry,
  controller,
  selectable,
  onContextMenu,
}: {
  entry: FileEntry;
  controller: FileManagerController;
  selectable: boolean;
  onContextMenu: FileEntryContextMenuHandler;
}) {
  return (
    <FileEntryCardPaper
      entry={entry}
      controller={controller}
      selectable={selectable}
      onContextMenu={(event) => onContextMenu(event, entry)}
    />
  );
}

function FileEntryCardPaper({
  entry,
  controller,
  selectable,
  onContextMenu,
}: EntryViewProps & {
  selectable: boolean;
  onContextMenu: MouseEventHandler<HTMLDivElement>;
}) {
  return (
    <Paper
      variant="outlined"
      sx={{ p: 2, minWidth: 0 }}
      onContextMenu={onContextMenu}
      onDoubleClick={() => activateFileEntry(entry, controller)}
    >
      <Stack spacing={1.5}>
        <CardHeader entry={entry} controller={controller} selectable={selectable} />
        <CardBody entry={entry} controller={controller} />
      </Stack>
    </Paper>
  );
}

function CardHeader({ entry, controller, selectable }: EntryViewProps & { selectable: boolean }) {
  return (
    <Stack direction="row" alignItems="center" justifyContent="space-between">
      {selectable ? (
        <Checkbox
          checked={controller.state.table.selected.includes(entry.id)}
          onChange={() => controller.state.table.onSelectRow(entry.id)}
          onDoubleClick={(event) => event.stopPropagation()}
          inputProps={{ 'aria-label': entry.name }}
        />
      ) : (
        <Box sx={{ width: 42 }} />
      )}
      <EntryActions entry={entry} controller={controller} />
    </Stack>
  );
}

function CardBody({ entry, controller }: EntryViewProps) {
  const { t } = useTranslate('admin');
  const folder = entry.type === 'folder';
  return (
    <Box
      role="button"
      tabIndex={0}
      onClick={() => openFileEntryDetails(entry, controller)}
      onKeyDown={(event) => handleCardKeyDown(event, entry, controller)}
      sx={{ cursor: 'pointer' }}
    >
      <ManagedFileThumbnail
        entry={entry}
        enabled={onlineThumbnailEnabled(controller)}
        errorLabel={t('file.messages.thumbnailFailed')}
        sx={{ width: 1, height: 150 }}
      />
      <Typography noWrap variant="subtitle2" sx={{ mt: 1 }}>
        {entry.name}
      </Typography>
      <Typography variant="caption" color="text.secondary">
        {folder ? t('file.folder') : fData(entry.size_bytes)}
      </Typography>
    </Box>
  );
}

function handleCardKeyDown(
  event: KeyboardEvent<HTMLElement>,
  entry: FileEntry,
  controller: FileManagerController
) {
  if (event.key === 'Enter') activateFileEntry(entry, controller);
}

function onlineThumbnailEnabled(controller: FileManagerController) {
  return controller.state.mode !== 'trash' && controller.permissions.canQuery;
}
