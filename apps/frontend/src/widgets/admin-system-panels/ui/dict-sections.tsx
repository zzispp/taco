import type React from 'react';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { DictData, DictType } from 'src/entities/system';
import type { DictDataFiltersValue, DictTypeFiltersValue } from './dict-constants';
import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

import { DictDataToolbar } from './dict-data-toolbar';
import { DictDataRow, DictTypeRow } from './dict-rows';
import { DictDataFilters, DictTypeFilters } from './dict-filters';

export function DictTypeSection(props: DictTypeSectionProps) {
  return (
    <Card>
      <DictTypeFilters
        filters={props.filters}
        error={props.filterError}
        onChange={props.onFilterChange}
      />
      <DictTypeTable props={props} />
      <TablePaginationCustom
        page={props.table.page}
        count={props.resource.total}
        rowsPerPage={props.table.rowsPerPage}
        onPageChange={props.table.onChangePage}
        onRowsPerPageChange={props.table.onChangeRowsPerPage}
      />
    </Card>
  );
}

function DictTypeTable({ props }: { props: DictTypeSectionProps }) {
  const { t } = useTranslate('admin');
  return (
    <Scrollbar>
      <Table sx={{ minWidth: 920 }}>
        <ManagementTableHead
          head={props.head}
          rowCount={props.resource.items.length}
          numSelected={props.selectedIds.length}
          onSelectAllRows={props.canRemove ? props.onToggleAll : undefined}
        />
        <TableBody>
          <DictTypeRows props={props} />
          <TableNoData
            title={t('common.noData')}
            notFound={!props.resource.isLoading && props.resource.items.length === 0}
          />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function DictTypeRows({ props }: { props: DictTypeSectionProps }) {
  if (props.resource.isLoading) {
    return <TableLoadingRows head={props.loadingHead} rows={props.table.rowsPerPage} />;
  }
  return props.resource.items.map((row) => (
    <DictTypeRow
      key={row.dict_id}
      row={row}
      selected={row.dict_type === props.activeType}
      checked={props.selectedIds.includes(row.dict_id)}
      canRemove={props.canRemove}
      onCheck={props.onToggleRow}
      onSelect={props.onSelect}
      onEdit={props.onEdit}
      onDelete={props.onDelete}
    />
  ));
}

export function DictDataSection(props: DictDataSectionProps) {
  return (
    <Card>
      <DictDataToolbar
        activeType={props.activeType}
        selectedCount={props.selectedIds.length}
        canAdd={props.canAdd}
        canRemove={props.canRemove}
        canExport={props.canExport}
        onAdd={props.onAdd}
        onBatchDelete={props.onBatchDelete}
        onExport={props.onExport}
      />
      <DictDataFilters filters={props.filters} onChange={props.onFilterChange} />
      <DictDataTable props={props} />
      <TablePaginationCustom
        page={props.table.page}
        count={props.resource.total}
        rowsPerPage={props.table.rowsPerPage}
        onPageChange={props.table.onChangePage}
        onRowsPerPageChange={props.table.onChangeRowsPerPage}
      />
    </Card>
  );
}

function DictDataTable({ props }: { props: DictDataSectionProps }) {
  const { t } = useTranslate('admin');
  return (
    <Scrollbar>
      <Table sx={{ minWidth: 1080 }}>
        <ManagementTableHead
          head={props.head}
          rowCount={props.resource.items.length}
          numSelected={props.selectedIds.length}
          onSelectAllRows={props.canRemove ? props.onToggleAll : undefined}
        />
        <TableBody>
          <DictDataRows props={props} />
          <TableNoData
            title={t('common.noData')}
            notFound={!props.resource.isLoading && props.resource.items.length === 0}
          />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function DictDataRows({ props }: { props: DictDataSectionProps }) {
  if (props.resource.isLoading) {
    return <TableLoadingRows head={props.loadingHead} rows={props.table.rowsPerPage} />;
  }
  return props.resource.items.map((row) => (
    <DictDataRow
      key={row.dict_code}
      row={row}
      selected={props.selectedIds.includes(row.dict_code)}
      canRemove={props.canRemove}
      onCheck={props.onToggleRow}
      onEdit={props.onEdit}
      onDelete={props.onDelete}
    />
  ));
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
  filterError: LocalDateTimeFilterError | null;
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
