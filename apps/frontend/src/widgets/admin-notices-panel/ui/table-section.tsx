import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { NoticeManagementController } from 'src/features/notice-management';

import Card from '@mui/material/Card';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { NoticeRow } from './row';

const HEAD: TableHeadCellProps[] = [
  { id: 'notice_title', label: 'notice.fields.title', width: 260 },
  { id: 'notice_type', label: 'notice.fields.type', width: 130 },
  { id: 'status', label: 'common.status', width: 110 },
  { id: 'create_by', label: 'notice.fields.createBy', width: 140 },
  { id: 'create_time', label: 'fields.createTime', width: 180 },
  { id: 'actions', label: 'common.actions', align: 'right', width: 180 },
];

export function NoticeTableSection({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state, resources, permissions } = controller;
  const head = HEAD.map((cell) => ({
    ...cell,
    label: cell.label ? t(cell.label) : '',
  }));
  const loadingHead = permissions.canRemove ? withSelectionHead(head) : head;
  if (resources.notices.error)
    return <Alert severity="error">{getErrorMessage(resources.notices.error)}</Alert>;
  return (
    <Card>
      <Scrollbar>
        <Table sx={{ minWidth: 1050 }}>
          <NoticeTableHeader controller={controller} head={head} />
          <NoticeTableBody controller={controller} head={loadingHead} />
        </Table>
      </Scrollbar>
      <CursorPagination
        limit={state.table.limit}
        itemCount={resources.notices.itemCount}
        visitedBatchIndex={state.table.visitedBatchIndex}
        hasPrevious={resources.notices.hasPrevious}
        hasNext={resources.notices.hasNext}
        pending={resources.notices.isValidating}
        onPrevious={() => state.table.onPreviousCursor(resources.notices.previousCursor)}
        onNext={() => state.table.onNextCursor(resources.notices.nextCursor)}
        onLimitChange={state.table.onChangeLimit}
      />
    </Card>
  );
}

function NoticeTableHeader({
  controller,
  head,
}: {
  controller: NoticeManagementController;
  head: TableHeadCellProps[];
}) {
  const { state, resources, permissions } = controller;
  return (
    <ManagementTableHead
      head={head}
      rowCount={resources.notices.items.length}
      numSelected={permissions.canRemove ? state.table.selected.length : 0}
      onSelectAllRows={
        permissions.canRemove
          ? (checked) =>
              state.table.onSelectAllRows(
                checked,
                resources.notices.items.map((notice) => notice.notice_id)
              )
          : undefined
      }
    />
  );
}

function NoticeTableBody({
  controller,
  head,
}: {
  controller: NoticeManagementController;
  head: TableHeadCellProps[];
}) {
  const { t } = useTranslate('admin');
  const { state, resources } = controller;
  return (
    <TableBody>
      {resources.notices.isLoading ? (
        <TableLoadingRows head={head} rows={state.table.limit} />
      ) : (
        resources.notices.items.map((notice) => (
          <NoticeRow key={notice.notice_id} notice={notice} controller={controller} />
        ))
      )}
      <TableNoData
        colSpan={head.length}
        title={t('common.noData')}
        notFound={!resources.notices.isLoading && resources.notices.items.length === 0}
      />
    </TableBody>
  );
}
