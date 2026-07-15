import type { UseTableReturn, TableHeadCellProps } from 'src/shared/ui/table';
import type {
  OnlineSession,
  useOnlineSessions,
  OnlineSessionFilters,
} from 'src/entities/online-session';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { TableLoadingRows, ManagementTableHead } from 'src/widgets/admin-common';

import { OnlineSessionRow } from './row';
import { OnlineSessionFiltersBar } from './filters';

const ONLINE_SESSIONS_TABLE_MIN_WIDTH = 1510;

export function OnlineSessionsTableSection({
  table,
  filters,
  sessions,
  head,
  loading,
  canForceLogout,
  filterErrorMessage,
  onFilterChange,
  onForceLogout,
}: OnlineSessionsTableSectionProps) {
  return (
    <Card>
      <OnlineSessionFiltersBar
        filters={filters}
        errorMessage={filterErrorMessage}
        onChange={onFilterChange}
      />
      <OnlineSessionsTable
        table={table}
        rows={sessions.rows}
        head={head}
        loading={loading}
        canForceLogout={canForceLogout}
        onForceLogout={onForceLogout}
      />
      <OnlineSessionsPagination table={table} sessions={sessions} />
    </Card>
  );
}

function OnlineSessionsTable({
  table,
  rows,
  head,
  loading,
  canForceLogout,
  onForceLogout,
}: Pick<
  OnlineSessionsTableSectionProps,
  'table' | 'head' | 'loading' | 'canForceLogout' | 'onForceLogout'
> & { rows: OnlineSession[] }) {
  const { t } = useTranslate('admin');
  return (
    <Scrollbar>
      <Table sx={{ minWidth: ONLINE_SESSIONS_TABLE_MIN_WIDTH }}>
        <ManagementTableHead head={head} />
        <TableBody>
          {loading ? (
            <TableLoadingRows head={head} rows={table.limit} />
          ) : (
            rows.map((row, rowIndex) => (
              <OnlineSessionRow
                key={row.tokenId}
                row={row}
                index={rowIndex + 1}
                canForceLogout={canForceLogout}
                onForceLogout={onForceLogout}
              />
            ))
          )}
          <TableNoData
            colSpan={head.length}
            title={t('common.noData')}
            notFound={!loading && rows.length === 0}
          />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function OnlineSessionsPagination({
  table,
  sessions,
}: Pick<OnlineSessionsTableSectionProps, 'table' | 'sessions'>) {
  return (
    <CursorPagination
      limit={table.limit}
      itemCount={sessions.itemCount}
      visitedBatchIndex={table.visitedBatchIndex}
      hasPrevious={sessions.hasPrevious}
      hasNext={sessions.hasNext}
      pending={sessions.isValidating}
      onPrevious={() => table.onPreviousCursor(sessions.previousCursor)}
      onNext={() => table.onNextCursor(sessions.nextCursor)}
      onLimitChange={table.onChangeLimit}
    />
  );
}

type OnlineSessionsTableSectionProps = {
  table: UseTableReturn;
  filters: OnlineSessionFilters;
  sessions: ReturnType<typeof useOnlineSessions>;
  head: TableHeadCellProps[];
  loading: boolean;
  canForceLogout: boolean;
  filterErrorMessage: string | null;
  onFilterChange: (filters: OnlineSessionFilters) => void;
  onForceLogout: (row: OnlineSession) => void;
};
