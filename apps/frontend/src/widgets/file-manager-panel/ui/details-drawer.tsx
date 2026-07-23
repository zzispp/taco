'use client';

import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import { useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Autocomplete from '@mui/material/Autocomplete';

import { Iconify } from 'src/shared/ui/iconify';
import { fData } from 'src/shared/lib/format-number';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';

import { ManagedFileThumbnail } from 'src/entities/file';

import {
  canEditFileEntry,
  canDeleteFileEntry,
  canPreviewFileEntry,
  canRestoreFileEntry,
  canDownloadFileEntry,
  canViewFileEntryDetails,
} from 'src/features/file-management';

export function FileDetailsDrawer({ controller }: { controller: FileManagerController }) {
  const entry = controller.resources.detail.data;
  const open = Boolean(
    controller.state.detailId && canViewFileEntryDetails(controller.permissions)
  );
  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={controller.actions.closeDetail}
      slotProps={{
        backdrop: { invisible: true },
        paper: { sx: { width: { xs: 1, sm: 420 } } },
      }}
    >
      {entry ? (
        <DetailsContent entry={entry} controller={controller} />
      ) : (
        <DetailsLoading controller={controller} />
      )}
    </Drawer>
  );
}

function DetailsLoading({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const resource = controller.resources.detail;
  return (
    <Stack spacing={2} sx={{ p: 3 }}>
      <Typography variant="h6">{t('file.details')}</Typography>
      {resource.isLoading ? <Typography color="text.secondary">...</Typography> : null}
      {resource.error ? <Typography color="error">{String(resource.error)}</Typography> : null}
    </Stack>
  );
}

function DetailsContent({
  entry,
  controller,
}: {
  entry: FileEntry;
  controller: FileManagerController;
}) {
  const { t } = useTranslate('admin');
  const [name, setName] = useState(entry.name);
  const [tags, setTags] = useState(entry.tags);
  const inTrash = controller.state.mode === 'trash';
  const canEdit = canEditFileEntry(controller.state.mode, controller.permissions);

  useEffect(() => {
    setName(entry.name);
    setTags(entry.tags);
  }, [entry]);

  return (
    <Stack sx={{ height: 1 }}>
      <DetailsHeader controller={controller} />
      <Box sx={{ flex: 1, overflow: 'auto' }}>
        <Stack spacing={2.5} sx={{ p: 3 }}>
          <ManagedFileThumbnail
            entry={entry}
            enabled={!inTrash}
            errorLabel={t('file.messages.thumbnailFailed')}
            sx={{ width: 1, height: 220 }}
          />
          <TextField
            fullWidth
            label={t('file.fields.name')}
            value={name}
            disabled={!canEdit}
            onChange={(event) => setName(event.target.value)}
            onBlur={() => controller.actions.renameEntry(entry, name)}
          />
          <TagEditor
            entry={entry}
            tags={tags}
            setTags={setTags}
            controller={controller}
            editable={canEdit}
          />
          <Properties entry={entry} />
          <FilePreviewControl entry={entry} controller={controller} />
        </Stack>
      </Box>
      <DetailsActions entry={entry} controller={controller} />
    </Stack>
  );
}

function FilePreviewControl({
  entry,
  controller,
}: {
  entry: FileEntry;
  controller: FileManagerController;
}) {
  const { t } = useTranslate('admin');
  if (canPreviewFileEntry(controller.state.mode, controller.permissions, entry)) {
    return (
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:eye-bold" />}
        disabled={controller.pending.has(`preview:${entry.id}`)}
        onClick={() => controller.actions.previewEntry(entry)}
      >
        {t('file.preview')}
      </Button>
    );
  }
  if (controller.state.mode === 'trash') return null;
  return (
    <Typography variant="body2" color="text.secondary">
      {t('file.contentUnavailable')}
    </Typography>
  );
}

function DetailsHeader({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" alignItems="center" spacing={1} sx={{ p: 2.5 }}>
      <Typography variant="h6" sx={{ flex: 1 }}>
        {t('file.info')}
      </Typography>
      <IconButton onClick={controller.actions.closeDetail} aria-label={t('file.actions.close')}>
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Stack>
  );
}

function TagEditor({
  entry,
  tags,
  setTags,
  controller,
  editable,
}: {
  entry: FileEntry;
  tags: string[];
  setTags: (tags: string[]) => void;
  controller: FileManagerController;
  editable: boolean;
}) {
  const { t } = useTranslate('admin');
  return (
    <Autocomplete
      multiple
      freeSolo
      options={entry.tags}
      value={tags}
      disabled={!editable}
      onChange={(_, value) => {
        const next = value.map((item) => item.trim()).filter(Boolean);
        setTags(next);
        controller.actions.updateTags(entry, next);
      }}
      renderInput={(params) => <TextField {...params} label={t('file.fields.tags')} />}
      slotProps={{ chip: { size: 'small' } }}
    />
  );
}

function Properties({ entry }: { entry: FileEntry }) {
  const { t } = useTranslate('admin');
  const properties = [
    [t('file.fields.type'), entry.mime_type ?? t(`file.${entry.type}`)],
    [t('file.fields.size'), entry.type === 'folder' ? '-' : fData(entry.size_bytes)],
    [t('file.fields.created'), fAdminDateTime(entry.created_at)],
    [t('file.fields.modified'), fAdminDateTime(entry.updated_at)],
    [t('file.fields.checksum'), entry.properties.checksum_sha256 ?? '-'],
    [t('file.fields.provider'), entry.properties.provider_key ?? '-'],
  ] as const;
  return (
    <Stack spacing={1}>
      <Typography variant="subtitle2">{t('file.properties')}</Typography>
      {properties.map(([label, value]) => (
        <Stack key={label} direction="row" spacing={2}>
          <Typography variant="caption" color="text.secondary" sx={{ width: 100 }}>
            {label}
          </Typography>
          <Typography variant="body2" sx={{ wordBreak: 'break-all' }}>
            {value}
          </Typography>
        </Stack>
      ))}
    </Stack>
  );
}

function DetailsActions({
  entry,
  controller,
}: {
  entry: FileEntry;
  controller: FileManagerController;
}) {
  const { t } = useTranslate('admin');
  const inTrash = controller.state.mode === 'trash';
  const canRestore = canRestoreFileEntry(controller.state.mode, controller.permissions);
  const canDownload = canDownloadFileEntry(controller.state.mode, controller.permissions);
  const canEdit = canEditFileEntry(controller.state.mode, controller.permissions);
  const canDelete = canDeleteFileEntry(controller.state.mode, controller.permissions);
  return (
    <>
      <Divider />
      <Stack direction="row" spacing={1} sx={{ p: 2.5 }}>
        {canRestore ? (
          <Button variant="outlined" onClick={() => controller.actions.restoreEntry(entry)}>
            {t('file.actions.restore')}
          </Button>
        ) : null}
        {canDownload && entry.type === 'file' ? (
          <Button variant="outlined" onClick={() => controller.actions.downloadEntry(entry)}>
            {t('file.actions.download')}
          </Button>
        ) : null}
        {canEdit ? (
          <Button
            variant="outlined"
            startIcon={<Iconify icon="eva:arrow-forward-fill" />}
            onClick={() => controller.actions.requestMove(entry)}
          >
            {t('file.actions.move')}
          </Button>
        ) : null}
        {canDelete ? (
          <Button
            color="error"
            variant="contained"
            onClick={() => controller.actions.requestDelete(entry)}
          >
            {inTrash ? t('file.actions.purge') : t('file.actions.delete')}
          </Button>
        ) : null}
      </Stack>
    </>
  );
}
