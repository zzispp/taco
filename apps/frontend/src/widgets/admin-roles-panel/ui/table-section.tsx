import type { TranslateFn } from 'src/shared/i18n';
import type { DEFAULT_FILTERS } from './constants';
import type { Role, useRoles } from 'src/entities/role';
import type { useTable, TableHeadCellProps } from 'src/shared/ui/table';
import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { updateRoleStatus } from 'src/features/role-management';

import { TableLoadingRows, ManagementTableHead } from 'src/widgets/admin-common';

import { RoleRow } from './role-row';
import { RoleFilters } from './filters';
import { toggle, showError } from './helpers';

type RoleTableSectionProps = {
  t: TranslateFn;
  table: ReturnType<typeof useTable>;
  filters: typeof DEFAULT_FILTERS;
  filterError: LocalDateTimeFilterError | null;
  roles: ReturnType<typeof useRoles>;
  head: TableHeadCellProps[];
  loadingHead: TableHeadCellProps[];
  selectableRoles: Role[];
  selected: string[];
  canDelete: boolean;
  onFilterChange: (filters: typeof DEFAULT_FILTERS) => void;
  onToggleAll: (checked: boolean) => void;
  onSelectedChange: (selected: string[]) => void;
  onEdit: (role: Role) => void;
  onDelete: (role: Role) => void;
  onBind: (role: Role, type: 'menus' | 'depts') => void;
  onUsers: (role: Role) => void;
};

export function RoleTableSection(props: RoleTableSectionProps) {
  const { table, filters, roles, head, selectableRoles, selected, canDelete, onFilterChange } =
    props;

  return (
    <Card>
      <RoleFilters filters={filters} error={props.filterError} onChange={onFilterChange} />
      <Scrollbar>
        <Table sx={{ minWidth: 1260 }}>
          <ManagementTableHead
            head={head}
            rowCount={selectableRoles.length}
            numSelected={selected.length}
            onSelectAllRows={canDelete ? props.onToggleAll : undefined}
          />
          <RoleRows {...props} />
        </Table>
      </Scrollbar>
      <CursorPagination
        limit={table.limit}
        itemCount={roles.itemCount}
        visitedBatchIndex={table.visitedBatchIndex}
        hasPrevious={roles.hasPrevious}
        hasNext={roles.hasNext}
        pending={roles.isValidating}
        onPrevious={() => table.onPreviousCursor(roles.previousCursor)}
        onNext={() => table.onNextCursor(roles.nextCursor)}
        onLimitChange={table.onChangeLimit}
      />
    </Card>
  );
}

function RoleRows(props: RoleTableSectionProps) {
  const { t, table, roles, loadingHead } = props;

  return (
    <TableBody>
      {roles.isLoading ? (
        <TableLoadingRows head={loadingHead} rows={table.limit} />
      ) : (
        roles.items.map((row) => <RoleDataRow key={row.role_id} row={row} {...props} />)
      )}
      <TableNoData
        colSpan={loadingHead.length}
        title={t('common.noData')}
        notFound={!roles.isLoading && roles.items.length === 0}
      />
    </TableBody>
  );
}

function RoleDataRow({ row, ...props }: RoleTableSectionProps & { row: Role }) {
  const { t, selected, onSelectedChange, onEdit, onDelete, onBind, onUsers } = props;

  return (
    <RoleRow
      row={row}
      selected={selected.includes(row.role_id)}
      onToggleSelected={(id) => onSelectedChange(toggle(selected, id))}
      onEdit={onEdit}
      onDelete={onDelete}
      onBind={onBind}
      onUsers={onUsers}
      onStatusChange={(status) =>
        updateRoleStatus(row.role_id, status)
          .then(() => props.table.onResetCursor())
          .catch(showError(t))
      }
    />
  );
}
