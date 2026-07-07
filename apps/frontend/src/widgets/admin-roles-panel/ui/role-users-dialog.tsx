'use client';

import type { Role, RoleUser } from 'src/entities/role';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import { useState, useCallback } from 'react';

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

import { toast } from 'src/shared/ui/snackbar';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/shared/ui/admin';

import { useRoleUsers, translatedRoleName } from 'src/entities/role';

import { deleteRoleUsers, assignRoleUsers } from 'src/features/role-management';

export function RoleUsersDialog({ role, onClose }: { role: Role | null; onClose: () => void }) {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 5 });
  const [allocated, setAllocated] = useState(true);
  const [username, setUsername] = useState('');
  const [phonenumber, setPhonenumber] = useState('');
  const [selected, setSelected] = useState<string[]>([]);
  const users = useRoleUsers(role?.role_id ?? null, table.page, table.rowsPerPage, {
    allocated,
    username,
    phonenumber,
  });
  const head = roleUsersHead(t);
  const loadingHead = withSelectionHead(head);
  const toggleAll = useCallback(
    (checked: boolean) => {
      setSelected(checked ? users.items.map((user) => user.user_id) : []);
    },
    [users.items]
  );

  const submit = useCallback(async () => {
    if (!role || selected.length === 0) return;
    try {
      if (allocated) await deleteRoleUsers(role.role_id, selected);
      else await assignRoleUsers(role.role_id, selected);
      toast.success(t('messages.saved'));
      setSelected([]);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [allocated, role, selected, t]);

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.authorizedUsers', { name: role ? translatedRoleName(role, t) : '' })}
      </DialogTitle>
      <DialogContent>
        <Tabs
          value={allocated ? 'allocated' : 'unallocated'}
          onChange={(_, value) => {
            setAllocated(value === 'allocated');
            setSelected([]);
          }}
          sx={{ mb: 2 }}
        >
          <Tab value="allocated" label={t('actions.allocatedUsers')} />
          <Tab value="unallocated" label={t('actions.unallocatedUsers')} />
        </Tabs>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ mb: 2 }}>
          <TextField
            fullWidth
            size="small"
            label={t('common.username')}
            value={username}
            onChange={(event) => setUsername(event.target.value)}
          />
          <TextField
            fullWidth
            size="small"
            label={t('fields.phone')}
            value={phonenumber}
            onChange={(event) => setPhonenumber(event.target.value)}
          />
        </Stack>
        <Scrollbar>
          <Table size="small" sx={{ minWidth: 760 }}>
            <ManagementTableHead
              head={head}
              rowCount={users.items.length}
              numSelected={selected.length}
              onSelectAllRows={toggleAll}
            />
            <TableBody>
              {users.isLoading ? (
                <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
              ) : (
                users.items.map((user) => (
                  <RoleUserRow
                    key={user.user_id}
                    user={user}
                    selected={selected}
                    onToggle={(id) => setSelected(toggle(selected, id))}
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
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" disabled={selected.length === 0} onClick={submit}>
          {allocated ? t('actions.cancelAuth') : t('actions.batchAuth')}
        </Button>
      </DialogActions>
    </Dialog>
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
