import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { LoginLogController } from 'src/features/audit-log-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { LoginLogRow } from './login-log-row';
import { LoginLogFiltersBar } from './filters';

const TABLE_MIN_WIDTH = 1760;

export const LOGIN_LOG_HEAD = [
  { id: 'info_id', label: 'fields.loginId', width: 220, sortable: false },
  { id: 'user_name', label: 'fields.username', width: 140 },
  { id: 'ipaddr', label: 'fields.loginIp', width: 150 },
  { id: 'login_location', label: 'fields.loginLocation', width: 180, sortable: false },
  { id: 'browser', label: 'fields.browser', width: 130, sortable: false },
  { id: 'os', label: 'fields.os', width: 180, sortable: false },
  { id: 'status', label: 'fields.loginStatus', width: 100 },
  { id: 'event_type', label: 'fields.eventType', width: 130, sortable: false },
  { id: 'msg', label: 'fields.message', width: 240, sortable: false },
  { id: 'login_time', label: 'fields.loginTime', width: 180 },
  { id: 'actions', label: 'table.actions', width: 72, align: 'right', sortable: false },
] as const;

export function LoginLogTableSection({ controller }: { controller: LoginLogController }) {
  return (
    <Card>
      <LoginLogFiltersBar controller={controller} />
      <LoginLogTable controller={controller} />
    </Card>
  );
}

function LoginLogTable({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { resources } = controller;
  const head: TableHeadCellProps[] = LOGIN_LOG_HEAD.map((cell) => ({
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
  return <LoginTableData controller={controller} head={head} />;
}

function LoginTableData({
  controller,
  head,
}: {
  controller: LoginLogController;
  head: TableHeadCellProps[];
}) {
  const { t } = useTranslate('audit');
  const { state, resources, actions } = controller;
  const selectable = resources.canRemove || resources.canUnlock;
  const loadingHead = selectable ? withSelectionHead(head) : head;
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
              selectable ? (checked) => selectLoginBatch(controller, checked) : undefined
            }
          />
          <TableBody>
            {resources.logs.isLoading ? (
              <TableLoadingRows head={loadingHead} rows={state.table.limit} />
            ) : (
              resources.logs.items.map((log) => (
                <LoginLogRow key={log.info_id} log={log} controller={controller} />
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
      <LoginPagination controller={controller} />
    </>
  );
}

function LoginPagination({ controller }: { controller: LoginLogController }) {
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

function selectLoginBatch(controller: LoginLogController, checked: boolean) {
  controller.state.table.onSelectAllRows(
    checked,
    controller.resources.logs.items.map((log) => log.info_id)
  );
}
