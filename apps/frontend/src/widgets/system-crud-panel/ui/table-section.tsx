import type { CrudPanelProps } from './types';
import type { SystemCrudController } from './controller';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

import { CrudFilters } from './filters';
import { TableActions } from './table-actions';
import { toggle, fieldCellSx, formFromRow, displayField } from './helpers';

type CrudRecord = Record<string, unknown>;

type CrudTableSectionProps<T extends CrudRecord, I extends CrudRecord> = {
  props: CrudPanelProps<T, I>;
  controller: SystemCrudController<T, I>;
};

export function CrudTableSection<T extends CrudRecord, I extends CrudRecord>({
  props,
  controller,
}: CrudTableSectionProps<T, I>) {
  const { state, permissions, selectableRows, head, actions } = controller;

  return (
    <Card>
      <CrudFilters filters={props.filters ?? []} values={props.filterValues ?? {}} onChange={props.onFilterChange} />
      <Scrollbar>
        <Table sx={{ minWidth: 980 }}>
          <ManagementTableHead
            head={head}
            rowCount={selectableRows.length}
            numSelected={state.selected.length}
            onSelectAllRows={permissions.hasBatchDelete ? actions.toggleAll : undefined}
          />
          <CrudTableBody props={props} controller={controller} />
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={props.page}
        count={props.resource.total}
        rowsPerPage={props.rowsPerPage}
        onPageChange={props.onPageChange}
        onRowsPerPageChange={props.onRowsPerPageChange}
      />
    </Card>
  );
}

function CrudTableBody<T extends CrudRecord, I extends CrudRecord>({
  props,
  controller,
}: CrudTableSectionProps<T, I>) {
  const { t, bodyHead } = controller;

  return (
    <TableBody>
      {props.resource.isLoading ? (
        <TableLoadingRows head={bodyHead} rows={props.rowsPerPage} />
      ) : (
        props.resource.items.map((row) => <CrudRow key={String(row[props.idKey])} row={row} props={props} controller={controller} />)
      )}
      <TableNoData title={t('common.noData')} notFound={!props.resource.isLoading && props.resource.items.length === 0} />
    </TableBody>
  );
}

type CrudRowProps<T extends CrudRecord, I extends CrudRecord> = CrudTableSectionProps<T, I> & { row: T };

function CrudRow<T extends CrudRecord, I extends CrudRecord>({ row, props, controller }: CrudRowProps<T, I>) {
  const { t, state, permissions } = controller;
  const rowId = String(row[props.idKey]);
  const isRowSelectable = props.isRowSelectable ?? (() => true);

  return (
    <TableRow hover>
      {permissions.hasBatchDelete && (
        <TableCell padding="checkbox">
          <Checkbox
            disabled={!isRowSelectable(row)}
            checked={state.selected.includes(rowId)}
            onChange={() => state.setSelected(toggle(state.selected, rowId))}
          />
        </TableCell>
      )}
      {props.fields.filter((field) => !field.hiddenInTable).map((field) => (
        <TableCell key={String(field.key)} sx={fieldCellSx(field)}>
          {displayField(row[field.key as keyof T], field, { yes: t('common.yes'), no: t('common.no') })}
        </TableCell>
      ))}
      <TableCell align="right">
        <TableActions
          permissionPrefix={props.permissionPrefix}
          extra={props.extraActions?.(row)}
          deleteDisabled={!isRowSelectable(row)}
          onEdit={() => {
            state.setForm(formFromRow<T, I>(row, props.fields));
            state.setEditing(row);
          }}
          onDelete={() => state.setDeleteTarget(row)}
        />
      </TableCell>
    </TableRow>
  );
}
