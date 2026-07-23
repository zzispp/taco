'use client';

import type { FileEntry } from 'src/entities/file';

import { useMemo, useState } from 'react';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import ButtonBase from '@mui/material/ButtonBase';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useTable, CursorPagination, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { usePermissionChecker } from 'src/entities/session';
import {
  useFileEntries,
  useFileOverview,
  ManagedFileImage,
  supportsFileThumbnail,
} from 'src/entities/file';

import { fileCapabilities } from 'src/features/file-management';

export type AvatarAssetPickerProps = Readonly<{
  selectedId: string | null;
  onSelect: (entry: FileEntry) => void;
}>;

export function AvatarAssetPicker({ selectedId, onSelect }: AvatarAssetPickerProps) {
  const resources = useAvatarAssetResources();
  return <AvatarAssetPickerContent {...{ resources, selectedId, onSelect }} />;
}

function useAvatarAssetResources() {
  const { t } = useTranslate('admin');
  const permissions = fileCapabilities(usePermissionChecker());
  const [search, setSearch] = useState('');
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT, scopeKey: search.trim() });
  const canQuery = permissions.canQuery && permissions.canList;
  const overview = useFileOverview(undefined, canQuery);
  const spaceId = overview.data?.space_id;
  const entries = useFileEntries(
    table.cursorRequest,
    { space_id: spaceId, trashed: false, search: search.trim() || undefined },
    canQuery && Boolean(spaceId)
  );
  const imageEntries = useMemo(
    () =>
      entries.items.filter((entry) => entry.space_id === spaceId && supportsFileThumbnail(entry)),
    [entries.items, spaceId]
  );
  return { t, table, search, setSearch, canQuery, overview, entries, imageEntries };
}

function AvatarAssetPickerContent({
  resources,
  selectedId,
  onSelect,
}: AvatarAssetPickerProps & { resources: ReturnType<typeof useAvatarAssetResources> }) {
  if (!resources.canQuery) {
    return <Alert severity="warning">{resources.t('file.permissionDenied')}</Alert>;
  }
  if (resources.overview.isLoading || resources.entries.isLoading) {
    return (
      <Stack alignItems="center" justifyContent="center" sx={{ minHeight: 180 }}>
        <CircularProgress size={28} />
      </Stack>
    );
  }
  if (resources.overview.error || resources.entries.error) {
    return (
      <Alert severity="error">
        {getErrorMessage(resources.overview.error || resources.entries.error) ||
          resources.t('profile.assetLoadFailed')}
      </Alert>
    );
  }
  return (
    <Stack spacing={2}>
      <TextField
        size="small"
        fullWidth
        label={resources.t('profile.assetSearch')}
        value={resources.search}
        onChange={(event) => resources.setSearch(event.target.value)}
      />
      {resources.imageEntries.length ? (
        <AssetGrid entries={resources.imageEntries} selectedId={selectedId} onSelect={onSelect} />
      ) : (
        <Alert severity="info">{resources.t('profile.noImageAssets')}</Alert>
      )}
      {resources.entries.isValidating && (
        <Typography variant="caption">{resources.t('common.loading')}</Typography>
      )}
      <AssetPagination resources={resources} />
    </Stack>
  );
}

function AssetPagination({ resources }: { resources: ReturnType<typeof useAvatarAssetResources> }) {
  return (
    <CursorPagination
      limit={resources.table.limit}
      itemCount={resources.entries.itemCount}
      visitedBatchIndex={resources.table.visitedBatchIndex}
      hasPrevious={resources.entries.hasPrevious}
      hasNext={resources.entries.hasNext}
      pending={resources.entries.isValidating}
      onPrevious={() => resources.table.onPreviousCursor(resources.entries.previousCursor)}
      onNext={() => resources.table.onNextCursor(resources.entries.nextCursor)}
      onLimitChange={resources.table.onChangeLimit}
    />
  );
}

function AssetGrid({
  entries,
  selectedId,
  onSelect,
}: AvatarAssetPickerProps & { entries: ReadonlyArray<FileEntry> }) {
  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(96px, 1fr))',
        gap: 1.5,
        maxHeight: 360,
        overflowY: 'auto',
        p: 0.5,
      }}
    >
      {entries.map((entry) => (
        <AssetOption
          key={entry.id}
          entry={entry}
          selected={entry.id === selectedId}
          onSelect={onSelect}
        />
      ))}
    </Box>
  );
}

function AssetOption({
  entry,
  selected,
  onSelect,
}: Readonly<{
  entry: FileEntry;
  selected: boolean;
  onSelect: (entry: FileEntry) => void;
}>) {
  const { t } = useTranslate('admin');
  return (
    <ButtonBase
      component="div"
      onClick={() => onSelect(entry)}
      aria-label={entry.name}
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'stretch',
        gap: 0.5,
        p: 0.75,
        border: 2,
        borderColor: selected ? 'primary.main' : 'divider',
        borderRadius: 1,
        textAlign: 'left',
      }}
    >
      <AssetOptionPreview entry={entry} selected={selected} />
      <Typography variant="caption" noWrap title={entry.name}>
        {entry.name || t('profile.unnamedAsset')}
      </Typography>
    </ButtonBase>
  );
}

function AssetOptionPreview({
  entry,
  selected,
}: Readonly<{ entry: FileEntry; selected: boolean }>) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ position: 'relative', aspectRatio: '1 / 1' }}>
      <ManagedFileImage
        entry={entry}
        errorLabel={t('file.messages.thumbnailFailed')}
        variant="rounded"
        sx={{ width: 1, height: 1, bgcolor: 'action.hover' }}
      >
        <Iconify icon="solar:gallery-wide-bold" width={28} />
      </ManagedFileImage>
      {selected ? <AssetSelectedMark /> : null}
    </Box>
  );
}

function AssetSelectedMark() {
  return (
    <Box
      sx={{
        position: 'absolute',
        right: 4,
        top: 4,
        display: 'grid',
        placeItems: 'center',
        width: 22,
        height: 22,
        borderRadius: '50%',
        bgcolor: 'primary.main',
        color: 'primary.contrastText',
      }}
    >
      <Iconify icon="eva:checkmark-fill" width={16} />
    </Box>
  );
}
