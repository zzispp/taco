import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { SystemLogController } from 'src/features/system-log-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { SystemLogFiltersBar } from './filters';
import { SystemLogRow } from './system-log-row';

export const SYSTEM_LOG_HEAD = [
  { id: 'log_id', label: 'fields.id', width: 260, sortable: false },
  { id: 'level', label: 'fields.level', width: 110, sortable: false },
  { id: 'target', label: 'fields.target', width: 180, sortable: false },
  { id: 'message', label: 'fields.message', width: 400, sortable: false },
  { id: 'occurred_at', label: 'fields.occurredAt', width: 190, sortable: false },
  { id: 'actions', label: 'table.actions', width: 110, align: 'right', sortable: false },
] as const;

export function SystemLogTableSection({ controller }: { controller: SystemLogController }) {
  return (
    <Card>
      <SystemLogFiltersBar controller={controller} />
      <SystemLogTable controller={controller} />
    </Card>
  );
}

function SystemLogTable({ controller }: { controller: SystemLogController }) {
  const { resources } = controller;
  if (resources.logs.error) return <SystemLogListError message={resources.listErrorMessage} />;
  return (
    <>
      <SystemLogResults controller={controller} />
      <SystemLogPagination controller={controller} />
    </>
  );
}

function SystemLogResults({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { resources } = controller;
  const heads = systemLogTableHeads(t, resources.canRemove);
  return (
    <Scrollbar sx={{ containerType: 'inline-size' }}>
      <Table sx={{ minWidth: 1260 }}>
        <ManagementTableHead
          head={heads.header}
          order="desc"
          orderBy="occurred_at"
          onSort={() => undefined}
          rowCount={resources.logs.items.length}
          numSelected={controller.state.table.selected.length}
          selectAllRowsLabel={t('table.selectAll')}
          onSelectAllRows={selectAllHandler(controller)}
        />
        <SystemLogTableBody controller={controller} head={heads.body} />
      </Table>
    </Scrollbar>
  );
}

function SystemLogTableBody({
  controller,
  head,
}: {
  controller: SystemLogController;
  head: TableHeadCellProps[];
}) {
  const { t } = useTranslate('systemLog');
  const { logs } = controller.resources;
  return (
    <TableBody>
      {logs.isLoading ? (
        <TableLoadingRows head={head} rows={controller.state.table.limit} />
      ) : (
        logs.items.map((log) => <SystemLogRow key={log.log_id} log={log} controller={controller} />)
      )}
      <TableNoData
        colSpan={head.length}
        title={t('table.noData')}
        notFound={!logs.isLoading && logs.items.length === 0}
        sx={{ position: 'sticky', left: 0, width: '100cqw' }}
      />
    </TableBody>
  );
}

function SystemLogPagination({ controller }: { controller: SystemLogController }) {
  const { logs } = controller.resources;
  return (
    <CursorPagination
      limit={controller.state.table.limit}
      itemCount={logs.itemCount}
      visitedBatchIndex={controller.state.table.visitedBatchIndex}
      hasPrevious={logs.hasPrevious}
      hasNext={logs.hasNext}
      pending={logs.isValidating}
      onPrevious={() => controller.state.table.onPreviousCursor(logs.previousCursor)}
      onNext={() => controller.state.table.onNextCursor(logs.nextCursor)}
      onLimitChange={controller.state.table.onChangeLimit}
    />
  );
}

function SystemLogListError({ message }: { message: string | null }) {
  return (
    <Alert severity="error" sx={{ mx: 2, mb: 2 }}>
      {message}
    </Alert>
  );
}

function translatedHead(t: (key: string) => string): TableHeadCellProps[] {
  return SYSTEM_LOG_HEAD.map((cell) => ({ ...cell, label: t(cell.label) }));
}

export function systemLogTableHeads(t: (key: string) => string, selectable: boolean) {
  const header = translatedHead(t);
  return { header, body: selectable ? withSelectionHead(header) : header };
}

function selectAllHandler(controller: SystemLogController) {
  if (!controller.resources.canRemove) return undefined;
  return (checked: boolean) =>
    controller.state.table.onSelectAllRows(
      checked,
      controller.resources.logs.items.map((log) => log.log_id)
    );
}
