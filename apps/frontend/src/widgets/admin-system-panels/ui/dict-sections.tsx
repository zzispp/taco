import type React from 'react';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { DictData, DictType } from 'src/entities/system';
import type { DictDataFiltersValue, DictTypeFiltersValue } from './dict-constants';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import TableBody from '@mui/material/TableBody';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

import { DictDataRow, DictTypeRow } from './dict-rows';
import { DictDataFilters, DictTypeFilters } from './dict-filters';

export function DictTypeSection({
  table,
  filters,
  resource,
  activeType,
  head,
  loadingHead,
  selectedIds,
  canRemove,
  onFilterChange,
  onToggleAll,
  onToggleRow,
  onSelect,
  onEdit,
  onDelete,
}: DictTypeSectionProps) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <DictTypeFilters filters={filters} onChange={onFilterChange} />
      <Scrollbar>
        <Table sx={{ minWidth: 920 }}>
          <ManagementTableHead
            head={head}
            rowCount={resource.items.length}
            numSelected={selectedIds.length}
            onSelectAllRows={canRemove ? onToggleAll : undefined}
          />
          <TableBody>
            {resource.isLoading ? (
              <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
            ) : (
              resource.items.map((row) => (
                <DictTypeRow
                  key={row.dict_id}
                  row={row}
                  selected={row.dict_type === activeType}
                  checked={selectedIds.includes(row.dict_id)}
                  canRemove={canRemove}
                  onCheck={onToggleRow}
                  onSelect={onSelect}
                  onEdit={onEdit}
                  onDelete={onDelete}
                />
              ))
            )}
            <TableNoData
              title={t('common.noData')}
              notFound={!resource.isLoading && resource.items.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={resource.total}
        rowsPerPage={table.rowsPerPage}
        onPageChange={table.onChangePage}
        onRowsPerPageChange={table.onChangeRowsPerPage}
      />
    </Card>
  );
}

export function DictDataSection({
  table,
  filters,
  resource,
  activeType,
  head,
  loadingHead,
  selectedIds,
  canAdd,
  canRemove,
  canExport,
  onFilterChange,
  onToggleAll,
  onToggleRow,
  onEdit,
  onDelete,
  onAdd,
  onBatchDelete,
  onExport,
}: DictDataSectionProps) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ p: 2 }}>
        <Typography variant="h6">
          {t('fields.dictData')}：{activeType || '-'}
        </Typography>
        <Stack direction="row" spacing={1}>
          {canExport && (
            <Button
              variant="outlined"
              startIcon={<Iconify icon="solar:export-bold" />}
              disabled={!activeType}
              onClick={onExport}
            >
              {t('actions.export')}
            </Button>
          )}
          {canRemove && (
            <Button
              variant="outlined"
              color="error"
              disabled={selectedIds.length === 0}
              onClick={onBatchDelete}
            >
              {t('common.delete')}
            </Button>
          )}
          {canAdd && (
            <Button
              variant="contained"
              startIcon={<Iconify icon="mingcute:add-line" />}
              disabled={!activeType}
              onClick={onAdd}
            >
              {t('actions.addDictData')}
            </Button>
          )}
        </Stack>
      </Stack>
      <DictDataFilters filters={filters} onChange={onFilterChange} />
      <Scrollbar>
        <Table sx={{ minWidth: 1080 }}>
          <ManagementTableHead
            head={head}
            rowCount={resource.items.length}
            numSelected={selectedIds.length}
            onSelectAllRows={canRemove ? onToggleAll : undefined}
          />
          <TableBody>
            {resource.isLoading ? (
              <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
            ) : (
              resource.items.map((row) => (
                <DictDataRow
                  key={row.dict_code}
                  row={row}
                  selected={selectedIds.includes(row.dict_code)}
                  canRemove={canRemove}
                  onCheck={onToggleRow}
                  onEdit={onEdit}
                  onDelete={onDelete}
                />
              ))
            )}
            <TableNoData
              title={t('common.noData')}
              notFound={!resource.isLoading && resource.items.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={table.page}
        count={resource.total}
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

type Resource<T> = {
  items: T[];
  total: number;
  isLoading: boolean;
};

type DictTypeSectionProps = {
  table: TableState;
  filters: DictTypeFiltersValue;
  resource: Resource<DictType>;
  activeType: string;
  head: TableHeadCellProps[];
  loadingHead: TableHeadCellProps[];
  selectedIds: string[];
  canRemove: boolean;
  onFilterChange: (filters: DictTypeFiltersValue) => void;
  onToggleAll: (checked: boolean) => void;
  onToggleRow: (id: string) => void;
  onSelect: (row: DictType) => void;
  onEdit: (row: DictType) => void;
  onDelete: (row: DictType) => void;
};

type DictDataSectionProps = {
  table: TableState;
  filters: DictDataFiltersValue;
  resource: Resource<DictData>;
  activeType: string;
  head: TableHeadCellProps[];
  loadingHead: TableHeadCellProps[];
  selectedIds: string[];
  canAdd: boolean;
  canRemove: boolean;
  canExport: boolean;
  onFilterChange: (filters: DictDataFiltersValue) => void;
  onToggleAll: (checked: boolean) => void;
  onToggleRow: (id: string) => void;
  onEdit: (row: DictData) => void;
  onDelete: (row: DictData) => void;
  onAdd: () => void;
  onBatchDelete: () => void;
  onExport: () => void;
};
