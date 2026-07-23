import type { FileManagerController } from 'src/features/file-management';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import TableContainer from '@mui/material/TableContainer';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { EmptyContent } from 'src/shared/ui/empty-content';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, TableHeadCustom, CursorPagination } from 'src/shared/ui/table';

import { canSelectFileEntries } from 'src/features/file-management';

import { FileEntryRow, FileEntryCard } from './entry-item';
import { FileManagerContextMenus, useFileManagerContextMenu } from './file-manager-context-menu';

export function FileEntryCollection({ controller }: { controller: FileManagerController }) {
  const { t } = useTranslate('admin');
  const { entries } = controller.resources;
  const contextMenu = useFileManagerContextMenu(controller);
  if (!controller.permissions.canList) {
    return <Alert severity="warning">{t('file.permissionDenied')}</Alert>;
  }
  if (entries.isLoading) return <LoadingScreen portal={false} sx={{ minHeight: 240 }} />;
  if (entries.error) {
    return (
      <Alert severity="error">
        {getErrorMessage(entries.error) || t('file.messages.listFailed')}
      </Alert>
    );
  }
  if (!entries.items.length) {
    return (
      <>
        <Box onContextMenu={contextMenu.onDirectoryContextMenu}>
          <EmptyContent filled title={t('file.empty')} sx={{ py: 12 }} />
        </Box>
        <FileManagerContextMenus
          controller={controller}
          target={contextMenu.target}
          onClose={contextMenu.close}
        />
      </>
    );
  }
  return (
    <>
      {controller.state.viewMode === 'grid' ? (
        <GridEntries controller={controller} contextMenu={contextMenu} />
      ) : (
        <ListEntries controller={controller} contextMenu={contextMenu} />
      )}
      <FileManagerContextMenus
        controller={controller}
        target={contextMenu.target}
        onClose={contextMenu.close}
      />
    </>
  );
}

type FileManagerContextMenu = ReturnType<typeof useFileManagerContextMenu>;

function GridEntries({
  controller,
  contextMenu,
}: {
  controller: FileManagerController;
  contextMenu: FileManagerContextMenu;
}) {
  const selectable = canSelectFileEntries(controller.state.mode, controller.permissions);
  return (
    <>
      <Box
        onContextMenu={contextMenu.onDirectoryContextMenu}
        sx={{
          display: 'grid',
          gap: 2,
          gridTemplateColumns: {
            xs: 'repeat(1, minmax(0, 1fr))',
            sm: 'repeat(2, minmax(0, 1fr))',
            md: 'repeat(3, minmax(0, 1fr))',
            lg: 'repeat(4, minmax(0, 1fr))',
          },
        }}
      >
        {controller.resources.entries.items.map((entry) => (
          <FileEntryCard
            key={entry.id}
            entry={entry}
            controller={controller}
            selectable={selectable}
            onContextMenu={contextMenu.onEntryContextMenu}
          />
        ))}
      </Box>
      <FilePagination controller={controller} />
    </>
  );
}

function ListEntries({
  controller,
  contextMenu,
}: {
  controller: FileManagerController;
  contextMenu: FileManagerContextMenu;
}) {
  const { t } = useTranslate('admin');
  const selectable = canSelectFileEntries(controller.state.mode, controller.permissions);
  return (
    <>
      <Scrollbar onContextMenu={contextMenu.onDirectoryContextMenu}>
        <TableContainer onContextMenu={contextMenu.onDirectoryContextMenu}>
          <Table sx={{ minWidth: 760 }}>
            <FileEntryTableHeader controller={controller} selectable={selectable} t={t} />
            <FileEntryTableRows
              controller={controller}
              contextMenu={contextMenu}
              selectable={selectable}
            />
          </Table>
        </TableContainer>
      </Scrollbar>
      <FilePagination controller={controller} />
    </>
  );
}

function FileEntryTableHeader({
  controller,
  selectable,
  t,
}: Readonly<{
  controller: FileManagerController;
  selectable: boolean;
  t: ReturnType<typeof useTranslate>['t'];
}>) {
  const table = controller.state.table;
  const ids = controller.resources.entries.items.map((entry) => entry.id);
  return (
    <TableHeadCustom
      headCells={fileEntryHead(t)}
      rowCount={ids.length}
      numSelected={selectable ? table.selected.length : 0}
      onSelectAllRows={selectable ? (checked) => table.onSelectAllRows(checked, ids) : undefined}
      onSort={table.onSort}
      order={table.order}
      orderBy={table.orderBy}
      selectAllRowsLabel={t('actions.selectAll')}
      sortStatusLabel={(order) => `${order}`}
    />
  );
}

function FileEntryTableRows({
  controller,
  contextMenu,
  selectable,
}: Readonly<{
  controller: FileManagerController;
  contextMenu: FileManagerContextMenu;
  selectable: boolean;
}>) {
  const table = controller.state.table;
  return (
    <TableBody>
      {controller.resources.entries.items.map((entry) => (
        <FileEntryRow
          key={entry.id}
          entry={entry}
          controller={controller}
          selected={table.selected.includes(entry.id)}
          selectable={selectable}
          onContextMenu={contextMenu.onEntryContextMenu}
        />
      ))}
      <TableNoData notFound={false} colSpan={selectable ? 6 : 5} />
    </TableBody>
  );
}

function fileEntryHead(t: ReturnType<typeof useTranslate>['t']) {
  return [
    { id: 'name', label: t('file.fields.name') },
    { id: 'type', label: t('file.fields.type'), width: 130, sortable: false },
    { id: 'size_bytes', label: t('file.fields.size'), width: 130, sortable: false },
    { id: 'updated_at', label: t('file.fields.modified'), width: 190 },
    { id: 'actions', label: '', width: 120, sortable: false },
  ];
}

function FilePagination({ controller }: { controller: FileManagerController }) {
  const table = controller.state.table;
  const entries = controller.resources.entries;
  return (
    <CursorPagination
      limit={table.limit}
      itemCount={entries.itemCount}
      visitedBatchIndex={table.visitedBatchIndex}
      hasPrevious={entries.hasPrevious}
      hasNext={entries.hasNext}
      pending={entries.isValidating}
      onPrevious={() => table.onPreviousCursor(entries.previousCursor)}
      onNext={() => table.onNextCursor(entries.nextCursor)}
      onLimitChange={table.onChangeLimit}
    />
  );
}
