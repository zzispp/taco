'use client';

import type { UploadQueueItem, UploadQueueTreeRow } from 'src/features/file-management';

import { useMemo } from 'react';
import { useDropzone } from 'react-dropzone';

import Box from '@mui/material/Box';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import LinearProgress from '@mui/material/LinearProgress';

import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { fData } from 'src/shared/lib/format-number';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  buildUploadQueueTree,
  truncateUploadDigest,
  uploadProgressPercent,
} from 'src/features/file-management';

const UPLOAD_QUEUE_MAX_HEIGHT = 300;
const UPLOAD_TREE_INDENTATION = 20;

type UploadQueueProps = Readonly<{
  items: readonly UploadQueueItem[];
  disabled: boolean;
  onAppend: (files: readonly File[]) => void;
  onRemove: (id: string) => void;
}>;

export function UploadQueue({ items, disabled, onAppend, onRemove }: UploadQueueProps) {
  const rows = useMemo(() => buildUploadQueueTree(items), [items]);
  return (
    <Stack spacing={2}>
      <UploadDropArea disabled={disabled} onAppend={onAppend} />
      {rows.length ? (
        <UploadQueueTable rows={rows} disabled={disabled} onRemove={onRemove} />
      ) : null}
    </Stack>
  );
}

function UploadDropArea({
  disabled,
  onAppend,
}: Readonly<Pick<UploadQueueProps, 'disabled' | 'onAppend'>>) {
  const { t } = useTranslate('admin');
  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    disabled,
    multiple: true,
    onDrop: (files) => onAppend(files),
  });
  return (
    <Box
      {...getRootProps()}
      sx={(theme) => ({
        cursor: disabled ? 'default' : 'pointer',
        border: 1,
        borderStyle: 'dashed',
        borderColor: isDragActive ? 'primary.main' : 'divider',
        bgcolor: isDragActive ? 'action.hover' : 'transparent',
        px: 2,
        py: 2.5,
        transition: theme.transitions.create(['background-color', 'border-color']),
      })}
    >
      <input {...getInputProps()} />
      <Stack direction="row" spacing={1.5} alignItems="center" justifyContent="center">
        <Iconify icon="eva:cloud-upload-fill" width={28} color="primary.main" />
        <Typography variant="body2" color="text.secondary">
          {isDragActive ? t('file.uploadDropActive') : t('file.uploadDropPrompt')}
        </Typography>
      </Stack>
    </Box>
  );
}

function UploadQueueTable({
  rows,
  disabled,
  onRemove,
}: Readonly<{
  rows: readonly UploadQueueTreeRow[];
  disabled: boolean;
  onRemove: (id: string) => void;
}>) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ border: 1, borderColor: 'divider' }}>
      <Stack
        direction="row"
        alignItems="center"
        sx={{ px: 2, py: 1.25, borderBottom: 1, borderColor: 'divider' }}
      >
        <Typography variant="subtitle2">{t('file.uploadQueue')}</Typography>
      </Stack>
      <Scrollbar sx={{ maxHeight: UPLOAD_QUEUE_MAX_HEIGHT }}>
        <Table size="small" sx={{ minWidth: 720 }} aria-label={t('file.uploadQueue')}>
          <TableHead>
            <TableRow>
              <TableCell>{t('file.fields.name')}</TableCell>
              <TableCell width={110}>{t('file.fields.size')}</TableCell>
              <TableCell width={150}>{t('file.fields.type')}</TableCell>
              <TableCell width={190}>{t('file.fields.checksum')}</TableCell>
              <TableCell width={160}>{t('common.status')}</TableCell>
              <TableCell width={48} />
            </TableRow>
          </TableHead>
          <TableBody>
            {rows.map((row) => (
              <UploadQueueRow key={row.id} row={row} disabled={disabled} onRemove={onRemove} />
            ))}
          </TableBody>
        </Table>
      </Scrollbar>
    </Box>
  );
}

function UploadQueueRow({
  row,
  disabled,
  onRemove,
}: Readonly<{
  row: UploadQueueTreeRow;
  disabled: boolean;
  onRemove: (id: string) => void;
}>) {
  const { t } = useTranslate('admin');
  const item = row.item;
  return (
    <TableRow hover aria-level={row.depth + 1}>
      <TableCell>
        <UploadQueueName row={row} />
      </TableCell>
      <TableCell>{fData(row.size)}</TableCell>
      <TableCell>
        {row.kind === 'folder' ? t('file.folder') : uploadItemType(item, t('file.file'))}
      </TableCell>
      <TableCell>{item ? <UploadDigest item={item} /> : '-'}</TableCell>
      <TableCell>{item ? <UploadItemStatus item={item} /> : '-'}</TableCell>
      <TableCell align="right">
        {item ? (
          <Tooltip title={t('common.delete')}>
            <span>
              <IconButton
                size="small"
                disabled={disabled}
                aria-label={t('common.delete')}
                onClick={() => onRemove(item.id)}
              >
                <Iconify icon="solar:trash-bin-trash-bold" width={18} />
              </IconButton>
            </span>
          </Tooltip>
        ) : null}
      </TableCell>
    </TableRow>
  );
}

function UploadQueueName({
  row,
}: Readonly<{
  row: Pick<UploadQueueTreeRow, 'depth' | 'kind' | 'name'>;
}>) {
  return (
    <Stack
      direction="row"
      spacing={1}
      alignItems="center"
      sx={{ minWidth: 220, pl: row.depth * UPLOAD_TREE_INDENTATION }}
    >
      <Iconify
        icon={row.kind === 'folder' ? 'solar:add-folder-bold' : 'solar:file-bold-duotone'}
        width={20}
      />
      <Typography noWrap variant="body2" title={row.name}>
        {row.name}
      </Typography>
    </Stack>
  );
}

function uploadItemType(item: UploadQueueItem | undefined, fallback: string): string {
  return item?.file.type || fallback;
}

function UploadDigest({ item }: { item: UploadQueueItem }) {
  const { t } = useTranslate('admin');
  if (!item.digest)
    return (
      <Typography variant="caption" color="text.secondary">
        {t(`file.uploadStates.${item.status}`)}
      </Typography>
    );
  return (
    <Tooltip title={item.digest} arrow>
      <Typography
        component="span"
        variant="caption"
        sx={{
          display: 'block',
          maxWidth: 170,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          fontFamily: 'monospace',
        }}
      >
        {truncateUploadDigest(item.digest)}
      </Typography>
    </Tooltip>
  );
}

function UploadItemStatus({ item }: { item: UploadQueueItem }) {
  const { t } = useTranslate('admin');
  const progress = item.progress;
  return (
    <Stack spacing={0.5}>
      <Typography
        variant="caption"
        color={item.status === 'failed' ? 'error.main' : 'text.secondary'}
      >
        {t(`file.uploadStates.${item.status}`)}
      </Typography>
      {progress ? (
        <LinearProgress variant="determinate" value={uploadProgressPercent(progress)} />
      ) : null}
    </Stack>
  );
}
