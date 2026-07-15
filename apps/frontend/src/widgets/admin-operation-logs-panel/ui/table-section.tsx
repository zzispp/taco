import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { OperationLogController } from 'src/features/audit-log-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { OperationLogFiltersBar } from './filters';
import { OperationLogRow } from './operation-log-row';

const TABLE_MIN_WIDTH = 2100;

export const OPERATION_LOG_HEAD = [
  { id: 'oper_id', label: 'fields.operationId', width: 220, sortable: false },
  { id: 'title', label: 'fields.title', width: 140, sortable: false },
  { id: 'business_type', label: 'fields.businessType', width: 120 },
  { id: 'oper_name', label: 'fields.operator', width: 130 },
  { id: 'dept_name', label: 'fields.department', width: 130, sortable: false },
  { id: 'request_method', label: 'fields.requestMethod', width: 110, sortable: false },
  { id: 'oper_url', label: 'fields.url', width: 220, sortable: false },
  { id: 'oper_ip', label: 'fields.operationIp', width: 150, sortable: false },
  { id: 'oper_location', label: 'fields.operationLocation', width: 180, sortable: false },
  { id: 'status', label: 'fields.operationStatus', width: 100 },
  { id: 'oper_time', label: 'fields.operationTime', width: 180 },
  { id: 'cost_time', label: 'fields.duration', width: 120 },
  { id: 'actions', label: 'table.actions', width: 110, align: 'right', sortable: false },
] as const;

export function OperationLogTableSection({ controller }: { controller: OperationLogController }) {
  return (
    <Card>
      <OperationLogFiltersBar controller={controller} />
      <OperationLogTable controller={controller} />
    </Card>
  );
}

function OperationLogTable({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { resources } = controller;
  const head: TableHeadCellProps[] = OPERATION_LOG_HEAD.map((cell) => ({
    ...cell,
    label: t(cell.label),
  }));
  if (resources.logs.error) {
    return (
      <Alert severity="error" sx={{ mx: 2, mb: 2 }}>
        {getErrorMessage(resources.logs.error)}
      </Alert>
    );
  }
  return <OperationTableData controller={controller} head={head} />;
}

function OperationTableData({
  controller,
  head,
}: {
  controller: OperationLogController;
  head: TableHeadCellProps[];
}) {
  const { t } = useTranslate('audit');
  const { state, resources, actions } = controller;
  const loadingHead = resources.canRemove ? withSelectionHead(head) : head;
  return (
    <>
      <Scrollbar sx={{ containerType: 'inline-size' }}>
        <Table sx={{ minWidth: TABLE_MIN_WIDTH }}>
          <ManagementTableHead
            head={head}
            order={state.table.order}
            orderBy={state.table.orderBy}
            onSort={actions.sort}
            rowCount={resources.logs.items.length}
            numSelected={state.table.selected.length}
            selectAllRowsLabel={t('table.selectAll')}
            onSelectAllRows={
              resources.canRemove
                ? (checked) => selectOperationBatch(controller, checked)
                : undefined
            }
          />
          <TableBody>
            {resources.logs.isLoading ? (
              <TableLoadingRows head={loadingHead} rows={state.table.limit} />
            ) : (
              resources.logs.items.map((log) => (
                <OperationLogRow key={log.oper_id} log={log} controller={controller} />
              ))
            )}
            <TableNoData
              colSpan={loadingHead.length}
              title={t('table.noData')}
              notFound={!resources.logs.isLoading && resources.logs.items.length === 0}
              sx={{ position: 'sticky', left: 0, width: '100cqw' }}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <OperationPagination controller={controller} />
    </>
  );
}

function OperationPagination({ controller }: { controller: OperationLogController }) {
  const { state, resources } = controller;
  return (
    <CursorPagination
      limit={state.table.limit}
      itemCount={resources.logs.itemCount}
      visitedBatchIndex={state.table.visitedBatchIndex}
      hasPrevious={resources.logs.hasPrevious}
      hasNext={resources.logs.hasNext}
      pending={resources.logs.isValidating}
      onPrevious={() => state.table.onPreviousCursor(resources.logs.previousCursor)}
      onNext={() => state.table.onNextCursor(resources.logs.nextCursor)}
      onLimitChange={state.table.onChangeLimit}
    />
  );
}

function selectOperationBatch(controller: OperationLogController, checked: boolean) {
  controller.state.table.onSelectAllRows(
    checked,
    controller.resources.logs.items.map((log) => log.oper_id)
  );
}
