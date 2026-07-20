'use client';

import type { useTranslate } from 'src/shared/i18n/use-locales';
import type { Role, RoleUser, useRoleUsers } from 'src/entities/role';
import type { useTable, TableHeadCellProps } from 'src/shared/ui/table';

import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { TableNoData, CursorPagination } from 'src/shared/ui/table';

import { translatedRoleName } from 'src/entities/role';

import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

type RoleUsersDialogContentProps = {
  role: Role | null;
  onClose: () => void;
  allocated: boolean;
  username: string;
  phonenumber: string;
  selected: string[];
  users: ReturnType<typeof useRoleUsers>;
  table: ReturnType<typeof useTable>;
  submit: () => Promise<void>;
  toggleAll: (checked: boolean) => void;
  t: ReturnType<typeof useTranslate>['t'];
  setAllocated: (allocated: boolean) => void;
  setUsername: (value: string) => void;
  setPhonenumber: (value: string) => void;
  setSelected: (selected: string[]) => void;
};

export function RoleUsersDialogContent({
  role,
  onClose,
  allocated,
  username,
  phonenumber,
  selected,
  users,
  table,
  submit,
  toggleAll,
  t,
  setAllocated,
  setUsername,
  setPhonenumber,
  setSelected,
}: RoleUsersDialogContentProps) {
  const head = roleUsersHead(t);

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.authorizedUsers', { name: role ? translatedRoleName(role) : '' })}
      </DialogTitle>
      <DialogContent>
        <RoleUserAllocationTabs
          allocated={allocated}
          onAllocatedChange={setAllocated}
          onSelectionClear={() => setSelected([])}
          allocatedLabel={t('actions.allocatedUsers')}
          unallocatedLabel={t('actions.unallocatedUsers')}
        />
        <RoleUserFilters
          username={username}
          phonenumber={phonenumber}
          onUsernameChange={setUsername}
          onPhonenumberChange={setPhonenumber}
          usernameLabel={t('common.username')}
          phoneLabel={t('fields.phone')}
        />
        <RoleUsersTable
          {...{ head, users, table, selected, onToggleAll: toggleAll }}
          onToggle={(id) => setSelected(toggle(selected, id))}
          noDataLabel={t('common.noData')}
        />
      </DialogContent>
      <RoleUsersDialogActions {...{ allocated, selected, onClose, submit, t }} />
    </Dialog>
  );
}

type RoleUsersDialogActionsProps = Pick<
  RoleUsersDialogContentProps,
  'allocated' | 'selected' | 'onClose' | 'submit' | 't'
>;

function RoleUsersDialogActions({
  allocated,
  selected,
  onClose,
  submit,
  t,
}: RoleUsersDialogActionsProps) {
  return (
    <DialogActions>
      <Button variant="outlined" onClick={onClose}>
        {t('common.cancel')}
      </Button>
      <Button variant="contained" disabled={selected.length === 0} onClick={submit}>
        {allocated ? t('actions.cancelAuth') : t('actions.batchAuth')}
      </Button>
    </DialogActions>
  );
}

function RoleUserAllocationTabs({
  allocated,
  onAllocatedChange,
  onSelectionClear,
  allocatedLabel,
  unallocatedLabel,
}: {
  allocated: boolean;
  onAllocatedChange: (allocated: boolean) => void;
  onSelectionClear: () => void;
  allocatedLabel: string;
  unallocatedLabel: string;
}) {
  return (
    <Tabs
      value={allocated ? 'allocated' : 'unallocated'}
      onChange={(_, value) => {
        onAllocatedChange(value === 'allocated');
        onSelectionClear();
      }}
      sx={{ mb: 2 }}
    >
      <Tab value="allocated" label={allocatedLabel} />
      <Tab value="unallocated" label={unallocatedLabel} />
    </Tabs>
  );
}

function RoleUserFilters({
  username,
  phonenumber,
  onUsernameChange,
  onPhonenumberChange,
  usernameLabel,
  phoneLabel,
}: {
  username: string;
  phonenumber: string;
  onUsernameChange: (value: string) => void;
  onPhonenumberChange: (value: string) => void;
  usernameLabel: string;
  phoneLabel: string;
}) {
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ mb: 2 }}>
      <TextField
        fullWidth
        size="small"
        label={usernameLabel}
        value={username}
        onChange={(event) => onUsernameChange(event.target.value)}
      />
      <TextField
        fullWidth
        size="small"
        label={phoneLabel}
        value={phonenumber}
        onChange={(event) => onPhonenumberChange(event.target.value)}
      />
    </Stack>
  );
}

type RoleUsersTableProps = {
  head: TableHeadCellProps[];
  users: ReturnType<typeof useRoleUsers>;
  table: ReturnType<typeof useTable>;
  selected: string[];
  onToggle: (id: string) => void;
  onToggleAll: (checked: boolean) => void;
  noDataLabel: string;
};

function RoleUsersTable(props: RoleUsersTableProps) {
  return (
    <>
      <RoleUsersDataTable {...props} />
      <RoleUsersPagination table={props.table} users={props.users} />
    </>
  );
}

function RoleUsersDataTable({
  head,
  users,
  table,
  selected,
  onToggle,
  onToggleAll,
  noDataLabel,
}: RoleUsersTableProps) {
  const loadingHead = withSelectionHead(head);

  return (
    <Scrollbar>
      <Table size="small" sx={{ minWidth: 760 }}>
        <ManagementTableHead
          head={head}
          rowCount={users.items.length}
          numSelected={selected.length}
          onSelectAllRows={onToggleAll}
        />
        <TableBody>
          {users.isLoading ? (
            <TableLoadingRows head={loadingHead} rows={table.limit} />
          ) : (
            users.items.map((user) => (
              <RoleUserRow key={user.user_id} user={user} selected={selected} onToggle={onToggle} />
            ))
          )}
          <TableNoData
            colSpan={loadingHead.length}
            title={noDataLabel}
            notFound={!users.isLoading && users.items.length === 0}
          />
        </TableBody>
      </Table>
    </Scrollbar>
  );
}

function RoleUsersPagination({ table, users }: Pick<RoleUsersTableProps, 'table' | 'users'>) {
  return (
    <CursorPagination
      limit={table.limit}
      itemCount={users.itemCount}
      visitedBatchIndex={table.visitedBatchIndex}
      hasPrevious={users.hasPrevious}
      hasNext={users.hasNext}
      pending={users.isValidating}
      onPrevious={() => table.onPreviousCursor(users.previousCursor)}
      onNext={() => table.onNextCursor(users.nextCursor)}
      onLimitChange={table.onChangeLimit}
    />
  );
}

function roleUsersHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'username', label: t('common.username') },
    { id: 'nick_name', label: t('fields.nickName') },
    { id: 'dept_name', label: t('fields.deptName') },
    { id: 'phonenumber', label: t('fields.phone') },
  ];
}

function RoleUserRow({
  user,
  selected,
  onToggle,
}: {
  user: RoleUser;
  selected: string[];
  onToggle: (id: string) => void;
}) {
  return (
    <TableRow hover>
      <TableCell padding="checkbox">
        <Checkbox
          checked={selected.includes(user.user_id)}
          onChange={() => onToggle(user.user_id)}
        />
      </TableCell>
      <TableCell>{user.username}</TableCell>
      <TableCell>{user.nick_name}</TableCell>
      <TableCell>{user.dept_name ?? '-'}</TableCell>
      <TableCell>{user.phonenumber ?? '-'}</TableCell>
    </TableRow>
  );
}

function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
