'use client';

import type { Role } from 'src/entities/role';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { UserInput, SystemUser } from 'src/entities/user';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  SwitchRow,
  TextFieldRow,
  EnabledLabel,
  BooleanLabel,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useRoles, translatedRoleName } from 'src/entities/role';
import { useUsers, translatedAuthSource } from 'src/entities/user';

import { createUser, deleteUser, updateUser } from 'src/features/user-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

// ----------------------------------------------------------------------

const DEFAULT_FORM: UserInput = {
  username: '',
  password: '',
  email: '',
  role: '',
  is_active: true,
};

// ----------------------------------------------------------------------

export function UserManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'username' });
  const { items, total, isLoading } = useUsers(table.page, table.rowsPerPage);
  const roles = useRoles(0, 100);
  const tableHead = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'username', label: t('common.username'), width: 220 },
      { id: 'email', label: t('common.email') },
      { id: 'role', label: t('common.role'), width: 160 },
      { id: 'auth_source', label: t('common.source'), width: 130 },
      { id: 'is_active', label: t('common.status'), width: 120 },
      { id: 'system', label: t('common.type'), width: 120 },
      { id: '', width: 96 },
    ],
    [t]
  );

  const [form, setForm] = useState<UserInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<SystemUser | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<SystemUser | null>(null);

  const roleOptions = roles.items.filter((role) => role.enabled);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({
      ...DEFAULT_FORM,
      role: roleOptions[0]?.code ?? '',
    });
  }, [roleOptions]);

  const openEdit = useCallback((user: SystemUser) => {
    setEditing(user);
    setForm({
      username: user.username,
      password: '',
      email: user.email,
      role: user.role,
      is_active: user.is_active,
    });
  }, []);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const submitUser = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateUser(editing.id, form);
        toast.success(t('messages.userUpdated'));
      } else {
        await createUser(form);
        toast.success(t('messages.userCreated'));
      }
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;

    try {
      await deleteUser(deleteTarget.id);
      toast.success(t('messages.userDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.userManagement')}
        action={<AddButton onClick={openCreate}>{t('actions.addUser')}</AddButton>}
      />

      <Card>
        <Scrollbar>
          <Table sx={{ minWidth: 980 }}>
            <ManagementTableHead head={tableHead} />
            <TableBody>
              {isLoading ? (
                <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
              ) : (
                items.map((row) => (
                  <TableRow key={row.id} hover>
                    <TableCell>{row.username}</TableCell>
                    <TableCell>{row.email}</TableCell>
                    <TableCell>{displayRole(row.role, roles.items, t)}</TableCell>
                    <TableCell>{translatedAuthSource(row.auth_source, t)}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.is_active} />
                    </TableCell>
                    <TableCell>
                      <BooleanLabel
                        enabled={row.system}
                        trueText={t('common.system')}
                        falseText={t('common.local')}
                      />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <Tooltip title={t('common.edit')}>
                          <span>
                            <IconButton disabled={row.system} onClick={() => openEdit(row)}>
                              <Iconify icon="solar:pen-bold" />
                            </IconButton>
                          </span>
                        </Tooltip>
                        <Tooltip title={t('common.delete')}>
                          <span>
                            <IconButton color="error" disabled={row.system} onClick={() => setDeleteTarget(row)}>
                              <Iconify icon="solar:trash-bin-trash-bold" />
                            </IconButton>
                          </span>
                        </Tooltip>
                      </Box>
                    </TableCell>
                  </TableRow>
                ))
              )}

              <TableNoData title={t('common.noData')} notFound={!isLoading && items.length === 0} />
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

      <ManagementDialog
        open={creating || !!editing}
        title={editing ? t('dialogs.editUser') : t('dialogs.createUser')}
        submitting={submitting}
        onClose={closeDialog}
        onSubmit={submitUser}
      >
        <TextFieldRow
          required
          label={t('common.username')}
          value={form.username}
          onChange={(value) => setForm((current) => ({ ...current, username: value }))}
        />
        <TextFieldRow
          required
          label={t('common.email')}
          value={form.email}
          onChange={(value) => setForm((current) => ({ ...current, email: value }))}
        />
        <TextFieldRow
          required
          select
          label={t('common.role')}
          value={form.role}
          onChange={(value) => setForm((current) => ({ ...current, role: value }))}
        >
          {roleOptions.map((role) => (
            <MenuItem key={role.code} value={role.code}>
              {translatedRoleName(role, t)} ({role.code})
            </MenuItem>
          ))}
        </TextFieldRow>
        <TextFieldRow
          required
          type="password"
          label={editing ? t('fields.newPassword') : t('common.password')}
          value={form.password}
          helperText={editing ? t('helper.updatePasswordRequired') : undefined}
          onChange={(value) => setForm((current) => ({ ...current, password: value }))}
        />
        <SwitchRow
          label={t('common.active')}
          checked={form.is_active}
          onChange={(isActive) => setForm((current) => ({ ...current, is_active: isActive }))}
        />
      </ManagementDialog>

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('dialogs.deleteUser')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.username ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

function displayRole(code: string, roles: Role[], t: ReturnType<typeof useTranslate>['t']) {
  const role = roles.find((item) => item.code === code);

  return role ? translatedRoleName(role, t) : code;
}
