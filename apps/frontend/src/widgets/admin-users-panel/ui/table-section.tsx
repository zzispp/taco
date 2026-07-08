import type React from 'react';
import type { FlatNode } from './helpers';
import type { TranslateFn } from 'src/shared/i18n';
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

const DEPT_FILTER_CARD_WIDTH = 280;
const USER_TABLE_MIN_WIDTH = 1560;

export function UserTableSection(props: UserTableSectionProps) {
  const { table, filters, users, roles, posts, deptTree, onDeptSelect, onFilterChange } = props;
  const { t } = useTranslate('admin');
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={3}>
      <Card
        sx={{
          width: { xs: 1, md: DEPT_FILTER_CARD_WIDTH },
          flexShrink: 0,
          alignSelf: 'flex-start',
        }}
      >
        <DeptFilterTree nodes={deptTree} selected={filters.dept_id} onSelect={onDeptSelect} />
      </Card>
      <Card sx={{ flex: 1, minWidth: 0 }}>
        <UserFilters filters={filters} roles={roles} posts={posts} onChange={onFilterChange} />
        <Scrollbar>
          <Table sx={{ minWidth: USER_TABLE_MIN_WIDTH }}>
            <UserTableContent props={{ ...props, t }} />
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

function UserTableContent({ props }: { props: UserTableContentProps }) {
  const selectAll = props.canDelete ? props.onToggleAll : undefined;
  return (
    <>
      <ManagementTableHead
        head={props.head}
        rowCount={props.selectableUsers.length}
        numSelected={props.selected.length}
        onSelectAllRows={selectAll}
      />
      <TableBody>
        <UserRows props={props} />
        <TableNoData
          title={props.t('common.noData')}
          notFound={!props.users.isLoading && props.users.items.length === 0}
        />
      </TableBody>
    </>
  );
}

function UserRows({ props }: { props: UserTableContentProps }) {
  if (props.users.isLoading) {
    return <TableLoadingRows head={props.loadingHead} rows={props.table.rowsPerPage} />;
  }
  return props.users.items.map((row) => (
    <UserRow
      key={row.user_id}
      row={row}
      selected={props.selected.includes(row.user_id)}
      roles={props.roles}
      depts={props.depts}
      posts={props.posts}
      onToggleSelected={props.onToggleSelected}
      onEdit={props.onEdit}
      onDelete={props.onDelete}
      onRoles={props.onRoles}
      onResetPassword={props.onResetPassword}
      onStatusChange={(status) => props.onStatusChange(row, status)}
    />
  ));
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

type UserTableContentProps = UserTableSectionProps & {
  t: TranslateFn;
};
