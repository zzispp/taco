import type React from 'react';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { OnlineSession, OnlineSessionFilters } from 'src/entities/online-session';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

import { OnlineSessionRow } from './row';
import { OnlineSessionFiltersBar } from './filters';

const ONLINE_SESSIONS_TABLE_MIN_WIDTH = 1510;

export function OnlineSessionsTableSection({
  table,
  filters,
  rows,
  total,
  head,
  loading,
  canForceLogout,
  onFilterChange,
  onForceLogout,
}: OnlineSessionsTableSectionProps) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <OnlineSessionFiltersBar filters={filters} onChange={onFilterChange} />
      <Scrollbar>
        <Table sx={{ minWidth: ONLINE_SESSIONS_TABLE_MIN_WIDTH }}>
          <ManagementTableHead head={head} />
          <TableBody>
            {loading ? (
              <TableLoadingRows head={head} rows={table.rowsPerPage} />
            ) : (
              rows.map((row, rowIndex) => (
                <OnlineSessionRow
                  key={row.tokenId}
                  row={row}
                  index={table.page * table.rowsPerPage + rowIndex + 1}
                  canForceLogout={canForceLogout}
                  onForceLogout={onForceLogout}
                />
              ))
            )}
            <TableNoData title={t('common.noData')} notFound={!loading && rows.length === 0} />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </Card>
  );
}

type TableState = {
  page: number;
  rowsPerPage: number;
  onChangePage: (event: unknown, page: number) => void;
  onChangeRowsPerPage: (event: React.ChangeEvent<HTMLInputElement>) => void;
};

type OnlineSessionsTableSectionProps = {
  table: TableState;
  filters: OnlineSessionFilters;
  rows: OnlineSession[];
  total: number;
  head: TableHeadCellProps[];
  loading: boolean;
  canForceLogout: boolean;
  onFilterChange: (filters: OnlineSessionFilters) => void;
  onForceLogout: (row: OnlineSession) => void;
};
