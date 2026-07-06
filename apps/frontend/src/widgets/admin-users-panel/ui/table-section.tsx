import type React from 'react';
import type { FlatNode } from './helpers';
import type { RoleOption } from 'src/entities/role';
import type { SystemUser } from 'src/entities/user';
import type { UserFiltersValue } from './constants';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { Post, TreeSelectNode } from 'src/entities/system';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

import { UserRow } from './user-row';
import { UserFilters } from './filters';
import { DeptFilterTree } from './dept-filter-tree';

export function UserTableSection({
  table,
  filters,
  users,
  roles,
  depts,
  posts,
  deptTree,
  head,
  loadingHead,
  selectableUsers,
  selected,
  canDelete,
  onFilterChange,
  onDeptSelect,
  onToggleAll,
  onToggleSelected,
  onEdit,
  onDelete,
  onRoles,
  onResetPassword,
  onStatusChange,
}: UserTableSectionProps) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={3}>
      <Card sx={{ width: { xs: 1, md: 280 }, flexShrink: 0, alignSelf: 'flex-start' }}>
        <DeptFilterTree nodes={deptTree} selected={filters.dept_id} onSelect={onDeptSelect} />
      </Card>
      <Card sx={{ flex: 1, minWidth: 0 }}>
        <UserFilters filters={filters} onChange={onFilterChange} />
        <Scrollbar>
          <Table sx={{ minWidth: 1560 }}>
            <ManagementTableHead
              head={head}
              rowCount={selectableUsers.length}
              numSelected={selected.length}
              onSelectAllRows={canDelete ? onToggleAll : undefined}
            />
            <TableBody>
              {users.isLoading ? (
                <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
              ) : (
                users.items.map((row) => (
                  <UserRow
                    key={row.user_id}
                    row={row}
                    selected={selected.includes(row.user_id)}
                    roles={roles}
                    depts={depts}
                    posts={posts}
                    onToggleSelected={onToggleSelected}
                    onEdit={onEdit}
                    onDelete={onDelete}
                    onRoles={onRoles}
                    onResetPassword={onResetPassword}
                    onStatusChange={(status) => onStatusChange(row, status)}
                  />
                ))
              )}
              <TableNoData
                title={t('common.noData')}
                notFound={!users.isLoading && users.items.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
        <TablePaginationCustom
          page={table.page}
          count={users.total}
          rowsPerPage={table.rowsPerPage}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
        />
      </Card>
    </Stack>
  );
}

type TableState = {
  page: number;
  rowsPerPage: number;
  onChangePage: (event: unknown, page: number) => void;
  onChangeRowsPerPage: (event: React.ChangeEvent<HTMLInputElement>) => void;
};

type UsersResource = {
  items: SystemUser[];
  total: number;
  isLoading: boolean;
};

type UserTableSectionProps = {
  table: TableState;
  filters: UserFiltersValue;
  users: UsersResource;
  roles: RoleOption[];
  depts: FlatNode[];
  posts: Post[];
  deptTree: TreeSelectNode[];
  head: TableHeadCellProps[];
  loadingHead: TableHeadCellProps[];
  selectableUsers: SystemUser[];
  selected: string[];
  canDelete: boolean;
  onFilterChange: (filters: UserFiltersValue) => void;
  onDeptSelect: (id: string) => void;
  onToggleAll: (checked: boolean) => void;
  onToggleSelected: (id: string) => void;
  onEdit: (user: SystemUser) => void;
  onDelete: (user: SystemUser) => void;
  onRoles: (user: SystemUser) => void;
  onResetPassword: (user: SystemUser) => void;
  onStatusChange: (user: SystemUser, status: string) => void;
};
