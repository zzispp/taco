'use client';

import { useMemo } from 'react';

import Table from '@mui/material/Table';
import Alert from '@mui/material/Alert';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import TableContainer from '@mui/material/TableContainer';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/shared/ui/iconify';
import { fData } from 'src/shared/lib/format-number';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useTable, TableNoData, TableHeadCustom, CursorPagination } from 'src/shared/ui/table';

import { usePermissionChecker } from 'src/entities/session';
import { useFileSpaces, type FileSpace, fileSpaceSortField } from 'src/entities/file';

import {
  MIN_QUOTA_GIB,
  QUOTA_STEP_GIB,
  fileCapabilities,
  useSpaceQuotaEditor,
  type SpaceQuotaEditor,
} from 'src/features/file-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { ProviderCapacityPanel } from './provider-capacity';

export function FileSpacePanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultOrderBy: 'updated_at', defaultOrder: 'desc' });
  const permissions = fileCapabilities(usePermissionChecker());
  const query = useMemo(
    () => ({ sort_by: fileSpaceSortField(table.orderBy), sort_order: table.order }),
    [table.order, table.orderBy]
  );
  const spaces = useFileSpaces(table.cursorRequest, query, permissions.canListSpaces);
  const quotaEditor = useSpaceQuotaEditor(permissions.canEditQuota);
  return (
    <DashboardContent maxWidth="xl">
      <AdminBreadcrumbs
        heading={t('file.spacesTitle')}
        parentLinks={[{ name: t('nav.fileManagement') }]}
      />
      <ProviderCapacityPanel enabled={permissions.canQueryProvider} />
      <SpaceTableState
        table={table}
        spaces={spaces}
        canList={permissions.canListSpaces}
        canEditQuota={permissions.canEditQuota}
        quotaEditor={quotaEditor}
      />
      <SpaceQuotaDialog editor={quotaEditor} />
    </DashboardContent>
  );
}

function SpaceTableState({
  table,
  spaces,
  canList,
  canEditQuota,
  quotaEditor,
}: Readonly<{
  table: ReturnType<typeof useTable>;
  spaces: ReturnType<typeof useFileSpaces>;
  canList: boolean;
  canEditQuota: boolean;
  quotaEditor: SpaceQuotaEditor;
}>) {
  const { t } = useTranslate('admin');
  if (!canList) return <Alert severity="warning">{t('file.permissionDenied')}</Alert>;
  if (spaces.isLoading) return <LoadingScreen portal={false} sx={{ minHeight: 280 }} />;
  if (spaces.error)
    return (
      <Alert severity="error">
        {getErrorMessage(spaces.error) || t('file.messages.spacesFailed')}
      </Alert>
    );
  return (
    <SpaceTable
      table={table}
      spaces={spaces}
      canEditQuota={canEditQuota}
      quotaEditor={quotaEditor}
    />
  );
}

function SpaceTable({
  table,
  spaces,
  canEditQuota,
  quotaEditor,
}: Readonly<{
  table: ReturnType<typeof useTable>;
  spaces: ReturnType<typeof useFileSpaces>;
  canEditQuota: boolean;
  quotaEditor: SpaceQuotaEditor;
}>) {
  const { t } = useTranslate('admin');
  const head = fileSpaceHead(t, canEditQuota);
  return (
    <>
      <TableContainer sx={{ border: 1, borderColor: 'divider', borderRadius: 1 }}>
        <Table sx={{ minWidth: 1040 }}>
          <TableHeadCustom
            headCells={head}
            order={table.order}
            orderBy={table.orderBy}
            onSort={table.onSort}
            selectAllRowsLabel={t('common.selectAll')}
            sortStatusLabel={(order) => order}
          />
          <SpaceRows
            spaces={spaces}
            canEditQuota={canEditQuota}
            onEditQuota={quotaEditor.open}
            colSpan={head.length}
          />
        </Table>
      </TableContainer>
      <CursorPagination
        limit={table.limit}
        itemCount={spaces.itemCount}
        visitedBatchIndex={table.visitedBatchIndex}
        hasPrevious={spaces.hasPrevious}
        hasNext={spaces.hasNext}
        pending={spaces.isValidating}
        onPrevious={() => table.onPreviousCursor(spaces.previousCursor)}
        onNext={() => table.onNextCursor(spaces.nextCursor)}
        onLimitChange={table.onChangeLimit}
      />
    </>
  );
}

function fileSpaceHead(t: ReturnType<typeof useTranslate>['t'], canEditQuota: boolean) {
  return [
    { id: 'owner_name', label: t('file.fields.owner') },
    { id: 'department_name', label: t('file.fields.department') },
    { id: 'status', label: t('file.fields.status'), width: 120 },
    { id: 'logical_asset_size', label: t('file.fields.usage'), width: 140 },
    { id: 'reserved_bytes', label: t('file.fields.reserved'), width: 140 },
    { id: 'quota_bytes', label: t('file.fields.quota'), width: 140 },
    { id: 'updated_at', label: t('file.fields.modified'), width: 190 },
    ...(canEditQuota ? [{ id: 'actions', label: '', width: 64 }] : []),
  ];
}

function SpaceRows({
  spaces,
  canEditQuota,
  onEditQuota,
  colSpan,
}: Readonly<{
  spaces: ReturnType<typeof useFileSpaces>;
  canEditQuota: boolean;
  onEditQuota: (space: FileSpace) => void;
  colSpan: number;
}>) {
  const { t } = useTranslate('admin');
  return (
    <TableBody>
      {spaces.items.map((space) => (
        <SpaceRow
          key={space.id}
          space={space}
          canEditQuota={canEditQuota}
          onEditQuota={onEditQuota}
        />
      ))}
      <TableNoData notFound={!spaces.items.length} colSpan={colSpan} title={t('file.noSpace')} />
    </TableBody>
  );
}

function SpaceRow({
  space,
  canEditQuota,
  onEditQuota,
}: {
  space: FileSpace;
  canEditQuota: boolean;
  onEditQuota: (space: FileSpace) => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <TableRow hover>
      <TableCell>{space.owner_name}</TableCell>
      <TableCell>{space.department_name ?? '-'}</TableCell>
      <TableCell>{t(`file.${space.status}`)}</TableCell>
      <TableCell>{fData(space.logical_asset_size)}</TableCell>
      <TableCell>{fData(space.reserved_bytes)}</TableCell>
      <TableCell>{fData(space.quota_bytes)}</TableCell>
      <TableCell>{fAdminDateTime(space.updated_at)}</TableCell>
      {canEditQuota && (
        <TableCell align="right">
          <Tooltip title={t('file.actions.editQuota')}>
            <IconButton
              size="small"
              aria-label={t('file.actions.editQuota')}
              onClick={() => onEditQuota(space)}
            >
              <Iconify icon="solar:pen-bold" width={18} />
            </IconButton>
          </Tooltip>
        </TableCell>
      )}
    </TableRow>
  );
}

function SpaceQuotaDialog({ editor }: { editor: SpaceQuotaEditor }) {
  const { t } = useTranslate('admin');
  const open = Boolean(editor.target);
  return (
    <Dialog fullWidth maxWidth="xs" open={open} onClose={editor.pending ? undefined : editor.close}>
      <DialogTitle>{t('file.quotaDialogTitle')}</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          fullWidth
          type="number"
          label={t('file.fields.quota')}
          value={editor.quotaGib}
          onChange={(event) => editor.setQuotaGib(event.target.value)}
          inputProps={{ min: MIN_QUOTA_GIB, step: QUOTA_STEP_GIB }}
          error={editor.quotaGib.length > 0 && editor.quotaBytes === null}
          InputProps={{
            endAdornment: <InputAdornment position="end">GiB</InputAdornment>,
          }}
          sx={{ mt: 1 }}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={editor.close} disabled={editor.pending}>
          {t('file.actions.cancel')}
        </Button>
        <Button onClick={editor.reset} disabled={editor.pending}>
          {t('file.actions.resetQuota')}
        </Button>
        <Button
          variant="contained"
          onClick={editor.save}
          disabled={editor.pending || editor.quotaBytes === null}
        >
          {t('file.actions.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
