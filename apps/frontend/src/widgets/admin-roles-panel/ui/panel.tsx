'use client';

import type { Role, RoleInput } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';
import type { TableHeadCellProps } from 'src/shared/ui/table';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  BooleanLabel,
  AdminBreadcrumbs,
  TableLoadingRows,
  withSelectionHead,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useHasPermission } from 'src/entities/session';
import { useRoles, translatedRoleName } from 'src/entities/role';

import {
  createRole,
  deleteRole,
  updateRole,
  exportRoles,
  deleteRoles,
  getRoleDeptTree,
  getRoleMenuTree,
  updateRoleMenus,
  updateRoleStatus,
  updateRoleDataScope,
} from 'src/features/role-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

import { RoleBindingDialog } from './binding-dialog';
import { RoleUsersDialog } from './role-users-dialog';
import { RoleDialog, dataScopeLabel } from './role-dialog';

const DEFAULT_FORM: RoleInput = {
  role_name: '',
  role_key: '',
  role_sort: 0,
  data_scope: '5',
  menu_check_strictly: true,
  dept_check_strictly: true,
  status: '0',
  remark: '',
};
const DEFAULT_FILTERS = { role_name: '', role_key: '', status: '', begin_time: '', end_time: '' };

export function RoleManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const roles = useRoles(table.page, table.rowsPerPage, filters);
  const [form, setForm] = useState<RoleInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [bindingTarget, setBindingTarget] = useState<Role | null>(null);
  const [bindingType, setBindingType] = useState<'menus' | 'depts'>('menus');
  const [selectedBindings, setSelectedBindings] = useState<string[]>([]);
  const [resolvedDeptBindings, setResolvedDeptBindings] = useState<string[]>([]);
  const [bindingNodes, setBindingNodes] = useState<TreeSelectNode[]>([]);
  const [bindingStrict, setBindingStrict] = useState(true);
  const [bindingDataScope, setBindingDataScope] = useState('5');
  const [bindingLoading, setBindingLoading] = useState(false);
  const [usersTarget, setUsersTarget] = useState<Role | null>(null);
  const head = useMemo(() => roleHead(t), [t]);
  const canAdd = useHasPermission('system:role:add');
  const canDelete = useHasPermission('system:role:remove');
  const canExport = useHasPermission('system:role:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableRoles = useMemo(() => roles.items.filter((role) => !role.system), [roles.items]);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);
  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_FORM);
  }, []);
  const openEdit = useCallback((role: Role) => {
    setEditing(role);
    setForm(toInput(role));
  }, []);
  const openBindings = useCallback(
    async (role: Role, type: 'menus' | 'depts') => {
      setBindingTarget(role);
      setBindingType(type);
      setResolvedDeptBindings([]);
      setBindingLoading(true);
      try {
        if (type === 'menus') {
          const data = await getRoleMenuTree(role.role_id);
          setBindingNodes(data.menus);
          setSelectedBindings(data.checked_keys);
          setBindingStrict(role.menu_check_strictly);
        } else {
          const data = await getRoleDeptTree(role.role_id);
          setBindingNodes(data.depts);
          setSelectedBindings(data.checked_keys);
          setBindingStrict(role.dept_check_strictly);
          setBindingDataScope(role.data_scope);
        }
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      } finally {
        setBindingLoading(false);
      }
    },
    [t]
  );

  const submitRole = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateRole(editing.role_id, form);
      else await createRole(form);
      toast.success(t('messages.saved'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, t]);

  const saveBindings = useCallback(async () => {
    if (!bindingTarget) return;
    setSubmitting(true);
    try {
      if (bindingType === 'menus') await updateRoleMenus(bindingTarget.role_id, resolvedDeptBindings);
      else
        await updateRoleDataScope(bindingTarget.role_id, {
          data_scope: bindingDataScope,
          dept_check_strictly: bindingStrict,
          dept_ids: bindingDataScope === '2' ? deptBindingIds(selectedBindings, resolvedDeptBindings, bindingStrict) : [],
        });
      toast.success(t('messages.rolePermissionsUpdated'));
      setBindingTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [bindingDataScope, bindingStrict, bindingTarget, bindingType, resolvedDeptBindings, selectedBindings, t]);

  const confirmDelete = useCallback(async () => {
    if (!deleteTarget) return;
    try {
      await deleteRole(deleteTarget.role_id);
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
      setSelected((current) => current.filter((id) => id !== deleteTarget.role_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  const confirmBatchDelete = useCallback(async () => {
    if (selected.length === 0) return;
    try {
      await deleteRoles(selected);
      toast.success(t('messages.deleted'));
      setSelected([]);
      setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [selected, t]);

  const toggleAll = useCallback(
    (checked: boolean) => {
      setSelected(checked ? selectableRoles.map((role) => role.role_id) : []);
    },
    [selectableRoles]
  );

  const submitExport = useCallback(async () => {
    try {
      await exportRoles(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [filters, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.roleManagement')}
        action={
          canDelete || canAdd || canExport ? (
            <Stack direction="row" spacing={1}>
              {canExport && (
                <Button
                  variant="outlined"
                  startIcon={<Iconify icon="solar:export-bold" />}
                  onClick={submitExport}
                >
                  {t('actions.export')}
                </Button>
              )}
              {canDelete && (
                <Button
                  variant="outlined"
                  color="error"
                  disabled={selected.length === 0}
                  onClick={() => setBatchDeleteOpen(true)}
                >
                  {t('common.delete')}
                </Button>
              )}
              {canAdd && <AddButton onClick={openCreate}>{t('actions.addRole')}</AddButton>}
            </Stack>
          ) : null
        }
      />
      <Card>
        <RoleFilters filters={filters} onChange={setFilters} />
        <Scrollbar>
          <Table sx={{ minWidth: 1260 }}>
            <ManagementTableHead
              head={head}
              rowCount={selectableRoles.length}
              numSelected={selected.length}
              onSelectAllRows={canDelete ? toggleAll : undefined}
            />
            <TableBody>
              {roles.isLoading ? (
                <TableLoadingRows head={loadingHead} rows={table.rowsPerPage} />
              ) : (
                roles.items.map((row) => (
                  <RoleRow
                    key={row.role_id}
                    row={row}
                    selected={selected.includes(row.role_id)}
                    onToggleSelected={(id) => setSelected(toggle(selected, id))}
                    onEdit={openEdit}
                    onDelete={setDeleteTarget}
                    onBind={openBindings}
                    onUsers={setUsersTarget}
                    onStatusChange={(status) =>
                      updateRoleStatus(row.role_id, status).catch(showError(t))
                    }
                  />
                ))
              )}
              <TableNoData
                title={t('common.noData')}
                notFound={!roles.isLoading && roles.items.length === 0}
              />
            </TableBody>
          </Table>
        </Scrollbar>
        <TablePaginationCustom
          page={table.page}
          count={roles.total}
          rowsPerPage={table.rowsPerPage}
          onPageChange={table.onChangePage}
          onRowsPerPageChange={table.onChangeRowsPerPage}
        />
      </Card>
      <RoleDialog
        open={creating || !!editing}
        editing={!!editing}
        submitting={submitting}
        form={form}
        setForm={setForm}
        onClose={closeDialog}
        onSubmit={submitRole}
      />
      <RoleBindingDialog
        role={bindingTarget}
        type={bindingType}
        nodes={bindingNodes}
        selected={selectedBindings}
        strict={bindingStrict}
        dataScope={bindingDataScope}
        loading={bindingLoading}
        submitting={submitting}
        onSelectedChange={setSelectedBindings}
        onStrictChange={setBindingStrict}
        onDataScopeChange={setBindingDataScope}
        onResolvedSelectionChange={setResolvedDeptBindings}
        onClose={() => setBindingTarget(null)}
        onSubmit={saveBindings}
      />
      <RoleUsersDialog role={usersTarget} onClose={() => setUsersTarget(null)} />
      <ConfirmDialog
        open={batchDeleteOpen}
        onClose={() => setBatchDeleteOpen(false)}
        title={t('dialogs.deleteRole')}
        content={t('dialogs.deleteContent', { name: String(selected.length) })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmBatchDelete}>
            {t('common.delete')}
          </Button>
        }
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('dialogs.deleteRole')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.role_name ?? '' })}
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

function deptBindingIds(selected: string[], resolved: string[], strict: boolean) {
  return strict ? resolved : selected;
}

function RoleFilters({
  filters,
  onChange,
}: {
  filters: typeof DEFAULT_FILTERS;
  onChange: (filters: typeof DEFAULT_FILTERS) => void;
}) {
  const { t } = useTranslate('admin');
  const write = (key: keyof typeof DEFAULT_FILTERS, value: string) =>
    onChange({ ...filters, [key]: value });
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} sx={{ p: 2 }}>
      <TextField
        size="small"
        label={t('fields.roleName')}
        value={filters.role_name}
        onChange={(event) => write('role_name', event.target.value)}
      />
      <TextField
        size="small"
        label={t('fields.roleKey')}
        value={filters.role_key}
        onChange={(event) => write('role_key', event.target.value)}
      />
      <TextField
        select
        size="small"
        label={t('common.status')}
        value={filters.status}
        sx={{ minWidth: 140 }}
        onChange={(event) => write('status', event.target.value)}
      >
        <MenuItem value="">{t('common.all')}</MenuItem>
        <MenuItem value="0">{t('common.enabled')}</MenuItem>
        <MenuItem value="1">{t('common.disabled')}</MenuItem>
      </TextField>
      <TextField
        size="small"
        type="date"
        label={t('fields.beginTime')}
        value={filters.begin_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('begin_time', event.target.value)}
      />
      <TextField
        size="small"
        type="date"
        label={t('fields.endTime')}
        value={filters.end_time}
        InputLabelProps={{ shrink: true }}
        onChange={(event) => write('end_time', event.target.value)}
      />
      <Button variant="outlined" onClick={() => onChange(DEFAULT_FILTERS)}>
        {t('common.reset')}
      </Button>
    </Stack>
  );
}

function RoleRow({
  row,
  selected,
  onToggleSelected,
  onEdit,
  onDelete,
  onBind,
  onUsers,
  onStatusChange,
}: {
  row: Role;
  selected: boolean;
  onToggleSelected: (id: string) => void;
  onEdit: (role: Role) => void;
  onDelete: (role: Role) => void;
  onBind: (role: Role, type: 'menus' | 'depts') => void;
  onUsers: (role: Role) => void;
  onStatusChange: (status: string) => void;
}) {
  const { t } = useTranslate('admin');
  const canEdit = useHasPermission('system:role:edit');
  const canDelete = useHasPermission('system:role:remove');
  return (
    <TableRow hover>
      {canDelete && (
        <TableCell padding="checkbox">
          <Checkbox
            disabled={row.system}
            checked={selected}
            onChange={() => onToggleSelected(row.role_id)}
          />
        </TableCell>
      )}
      <TableCell>{translatedRoleName(row, t)}</TableCell>
      <TableCell sx={{ fontFamily: 'monospace' }}>{row.role_key}</TableCell>
      <TableCell>{row.role_sort}</TableCell>
      <TableCell>{dataScopeLabel(row.data_scope, t)}</TableCell>
      <TableCell>
        <Switch
          size="small"
          checked={row.status === '0'}
          disabled={row.system || !canEdit}
          onChange={(event) => onStatusChange(event.target.checked ? '0' : '1')}
        />
      </TableCell>
      <TableCell>
        <BooleanLabel
          enabled={row.system}
          trueText={t('common.system')}
          falseText={t('common.custom')}
        />
      </TableCell>
      <TableCell sx={DATE_TIME_CELL_SX}>{fAdminDateTime(row.create_time) || '-'}</TableCell>
      <TableCell align="right">
        <RoleActions
          system={row.system}
          canEdit={canEdit}
          canDelete={canDelete}
          onMenu={() => onBind(row, 'menus')}
          onDept={() => onBind(row, 'depts')}
          onUsers={() => onUsers(row)}
          onEdit={() => onEdit(row)}
          onDelete={() => onDelete(row)}
        />
      </TableCell>
    </TableRow>
  );
}

function RoleActions({
  system,
  canEdit,
  canDelete,
  onMenu,
  onDept,
  onUsers,
  onEdit,
  onDelete,
}: {
  system: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onMenu: () => void;
  onDept: () => void;
  onUsers: () => void;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('actions.menuPermissions')}>
        <IconButton disabled={!canEdit} onClick={onMenu}>
          <Iconify icon="solar:shield-keyhole-bold-duotone" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('actions.dataPermissions')}>
        <IconButton disabled={!canEdit} onClick={onDept}>
          <Iconify icon="solar:notes-bold-duotone" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('actions.authorizedUsers')}>
        <IconButton disabled={!canEdit} onClick={onUsers}>
          <Iconify icon="solar:user-id-bold" />
        </IconButton>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={system || !canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={system || !canDelete} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}

function toInput(role: Role): RoleInput {
  return {
    role_name: role.role_name,
    role_key: role.role_key,
    role_sort: role.role_sort,
    data_scope: role.data_scope,
    menu_check_strictly: role.menu_check_strictly,
    dept_check_strictly: role.dept_check_strictly,
    status: role.status,
    remark: role.remark,
  };
}
function showError(t: ReturnType<typeof useTranslate>['t']) {
  return (error: unknown) =>
    toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
}
const TABLE_HEAD_SX = { whiteSpace: 'nowrap' } as const;
const DATE_TIME_CELL_SX = { whiteSpace: 'nowrap' } as const;
function roleHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'role_name', label: t('fields.roleName') },
    { id: 'role_key', label: t('fields.roleKey') },
    { id: 'role_sort', label: t('fields.roleSort') },
    { id: 'data_scope', label: t('fields.dataScope') },
    { id: 'status', label: t('common.status') },
    { id: 'system', label: t('common.type') },
    { id: 'create_time', label: t('fields.createTime'), width: 190, sx: TABLE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 220, sx: TABLE_HEAD_SX },
  ];
}
function toggle(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
