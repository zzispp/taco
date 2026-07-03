'use client';

import type { Role, RoleInput } from 'src/entities/role';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { ApiPermission } from 'src/entities/api-permission';
import type { MenuItem as RbacMenuItem } from 'src/entities/menu';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import List from '@mui/material/List';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import ListItem from '@mui/material/ListItem';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import ListItemText from '@mui/material/ListItemText';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ListItemButton from '@mui/material/ListItemButton';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  SwitchRow,
  MethodLabel,
  TextFieldRow,
  EnabledLabel,
  BooleanLabel,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import { useMenuItems, translatedMenuItem } from 'src/entities/menu';
import {
  useRoles,
  translatedRoleName,
  translatedRoleDescription,
} from 'src/entities/role';
import {
  useApis,
  translatedApiName,
  translatedApiGroup,
} from 'src/entities/api-permission';

import {
  createRole,
  deleteRole,
  updateRole,
  getRoleApis,
  getRoleMenus,
  updateRoleApis,
  updateRoleMenus,
} from 'src/features/role-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

// ----------------------------------------------------------------------

const DEFAULT_FORM: RoleInput = {
  code: '',
  name: '',
  description: '',
  enabled: true,
  sort_order: 0,
};

// ----------------------------------------------------------------------

export function RoleManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const { items, total, isLoading } = useRoles(table.page, table.rowsPerPage);
  const apis = useApis(0, 100);
  const menuItems = useMenuItems(0, 100);
  const tableHead = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'name', label: t('common.role'), width: 220 },
      { id: 'code', label: t('common.code'), width: 200 },
      { id: 'description', label: t('common.description') },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: 'system', label: t('common.type'), width: 120 },
      { id: '', width: 144 },
    ],
    [t]
  );

  const [form, setForm] = useState<RoleInput>(DEFAULT_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [bindingTarget, setBindingTarget] = useState<Role | null>(null);
  const [bindingTab, setBindingTab] = useState<'apis' | 'menus'>('apis');
  const [selectedApis, setSelectedApis] = useState<string[]>([]);
  const [selectedMenus, setSelectedMenus] = useState<string[]>([]);
  const [bindingLoading, setBindingLoading] = useState(false);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM });
  }, []);

  const openEdit = useCallback((role: Role) => {
    setEditing(role);
    setForm({
      code: role.code,
      name: role.name,
      description: role.description,
      enabled: role.enabled,
      sort_order: role.sort_order,
    });
  }, []);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const submitRole = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) {
        await updateRole(editing.code, form);
        toast.success(t('messages.roleUpdated'));
      } else {
        await createRole(form);
        toast.success(t('messages.roleCreated'));
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
      await deleteRole(deleteTarget.code);
      toast.success(t('messages.roleDeleted'));
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  const openBindings = useCallback(async (role: Role) => {
    setBindingTarget(role);
    setBindingLoading(true);
    setBindingTab('apis');
    try {
      const [apiBinding, menuBinding] = await Promise.all([
        getRoleApis(role.code),
        getRoleMenus(role.code),
      ]);
      setSelectedApis(apiBinding.api_permission_ids);
      setSelectedMenus(menuBinding.menu_item_ids);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
    } finally {
      setBindingLoading(false);
    }
  }, [t]);

  const saveBindings = useCallback(async () => {
    if (!bindingTarget) return;

    setSubmitting(true);
    try {
      await Promise.all([
        updateRoleApis(bindingTarget.code, selectedApis),
        updateRoleMenus(bindingTarget.code, selectedMenus),
      ]);
      toast.success(t('messages.rolePermissionsUpdated'));
      setBindingTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [bindingTarget, selectedApis, selectedMenus, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.roleManagement')}
        action={<AddButton onClick={openCreate}>{t('actions.addRole')}</AddButton>}
      />

      <Card>
        <Scrollbar>
          <Table sx={{ minWidth: 1050 }}>
            <ManagementTableHead head={tableHead} />
            <TableBody>
              {isLoading ? (
                <TableLoadingRows head={tableHead} rows={table.rowsPerPage} />
              ) : (
                items.map((row) => (
                  <TableRow key={row.code} hover>
                    <TableCell>{translatedRoleName(row, t)}</TableCell>
                    <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                    <TableCell>{translatedRoleDescription(row, t) || '-'}</TableCell>
                    <TableCell>{row.sort_order}</TableCell>
                    <TableCell>
                      <EnabledLabel enabled={row.enabled} />
                    </TableCell>
                    <TableCell>
                      <BooleanLabel
                        enabled={row.system}
                        trueText={t('common.system')}
                        falseText={t('common.custom')}
                      />
                    </TableCell>
                    <TableCell align="right">
                      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                        <Tooltip title={t('common.permissions')}>
                          <IconButton onClick={() => openBindings(row)}>
                            <Iconify icon="solar:shield-keyhole-bold-duotone" />
                          </IconButton>
                        </Tooltip>
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
        title={editing ? t('dialogs.editRole') : t('dialogs.createRole')}
        submitting={submitting}
        onClose={closeDialog}
        onSubmit={submitRole}
      >
        <TextFieldRow
          required
          disabled={!!editing}
          label={t('common.code')}
          value={form.code}
          onChange={(value) => setForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          required
          label={t('common.name')}
          value={form.name}
          onChange={(value) => setForm((current) => ({ ...current, name: value }))}
        />
        <TextFieldRow
          label={t('common.description')}
          value={form.description}
          onChange={(value) => setForm((current) => ({ ...current, description: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('common.sortOrder')}
          value={form.sort_order}
          onChange={(value) => setForm((current) => ({ ...current, sort_order: Number(value) }))}
        />
        <SwitchRow
          label={t('common.enabled')}
          checked={form.enabled}
          onChange={(enabled) => setForm((current) => ({ ...current, enabled }))}
        />
      </ManagementDialog>

      <BindingDialog
        role={bindingTarget}
        tab={bindingTab}
        loading={bindingLoading}
        submitting={submitting}
        apis={apis.items}
        menus={menuItems.items}
        selectedApis={selectedApis}
        selectedMenus={selectedMenus}
        onTabChange={setBindingTab}
        onSelectedApisChange={setSelectedApis}
        onSelectedMenusChange={setSelectedMenus}
        onClose={() => setBindingTarget(null)}
        onSubmit={saveBindings}
      />

      <ConfirmDialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t('dialogs.deleteRole')}
        content={t('dialogs.deleteContent', { name: deleteTarget?.name ?? '' })}
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

function BindingDialog({
  role,
  tab,
  loading,
  submitting,
  apis,
  menus,
  selectedApis,
  selectedMenus,
  onTabChange,
  onSelectedApisChange,
  onSelectedMenusChange,
  onClose,
  onSubmit,
}: {
  role: Role | null;
  tab: 'apis' | 'menus';
  loading: boolean;
  submitting: boolean;
  apis: ApiPermission[];
  menus: RbacMenuItem[];
  selectedApis: string[];
  selectedMenus: string[];
  onTabChange: (value: 'apis' | 'menus') => void;
  onSelectedApisChange: (value: string[]) => void;
  onSelectedMenusChange: (value: string[]) => void;
  onClose: () => void;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');

  const toggleApi = (id: string) => {
    onSelectedApisChange(toggleValue(selectedApis, id));
  };

  const toggleMenu = (id: string) => {
    onSelectedMenusChange(toggleValue(selectedMenus, id));
  };

  return (
    <Dialog fullWidth maxWidth="md" open={!!role} onClose={onClose}>
      <DialogTitle>
        {t('dialogs.rolePermissions', {
          name: role ? translatedRoleName(role, t) : '',
        })}
      </DialogTitle>
      <DialogContent>
        <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
          <Button
            variant={tab === 'apis' ? 'contained' : 'outlined'}
            onClick={() => onTabChange('apis')}
          >
            {t('actions.apiPermissions')}
          </Button>
          <Button
            variant={tab === 'menus' ? 'contained' : 'outlined'}
            onClick={() => onTabChange('menus')}
          >
            {t('actions.menuPermissions')}
          </Button>
        </Box>

        {loading ? (
          <Box sx={{ py: 4, color: 'text.secondary' }}>{t('messages.loadingPermissions')}</Box>
        ) : (
          <Scrollbar sx={{ maxHeight: 520 }}>
            {tab === 'apis' ? (
              <List disablePadding>
                {apis.map((api) => (
                  <ListItem key={api.id} disablePadding>
                    <ListItemButton onClick={() => toggleApi(api.id)}>
                      <Checkbox edge="start" checked={selectedApis.includes(api.id)} tabIndex={-1} />
                      <ListItemText
                        primary={
                          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                            <MethodLabel method={api.method} />
                            <span>{translatedApiName(api, t)}</span>
                          </Box>
                        }
                        secondary={`${translatedApiGroup(api.group, t)} · ${api.path_pattern}`}
                      />
                    </ListItemButton>
                  </ListItem>
                ))}
              </List>
            ) : (
              <List disablePadding>
                {menus.map((menu) => (
                  <ListItem key={menu.id} disablePadding>
                    <ListItemButton onClick={() => toggleMenu(menu.id)}>
                      <Checkbox edge="start" checked={selectedMenus.includes(menu.id)} tabIndex={-1} />
                      <ListItemText
                        primary={translatedMenuItem(menu, t)}
                        secondary={`${menu.code} · ${menu.path}`}
                      />
                    </ListItemButton>
                  </ListItem>
                ))}
              </List>
            )}
          </Scrollbar>
        )}
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onClose}>
          {t('common.cancel')}
        </Button>
        <Button variant="contained" loading={submitting} onClick={onSubmit}>
          {t('actions.savePermissions')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function toggleValue(values: string[], value: string) {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}
