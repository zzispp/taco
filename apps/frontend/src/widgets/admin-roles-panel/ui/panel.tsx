'use client';

import type { Role } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';

import { toast } from 'src/shared/ui/snackbar';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, withSelectionHead, ManagementTableHead } from 'src/shared/ui/admin';

import { useRoles } from 'src/entities/role';
import { useHasPermission } from 'src/entities/session';

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
} from 'src/features/role-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { RoleRow } from './role-row';
import { RoleFilters } from './filters';
import { RoleToolbar } from './toolbar';
import { RoleManagementDialogs } from './dialogs';
import { DEFAULT_FORM, DEFAULT_FILTERS } from './constants';
import { toggle, toInput, roleHead, showError, deptBindingIds } from './helpers';

export function RoleManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const roles = useRoles(table.page, table.rowsPerPage, filters);
  const [form, setForm] = useState(DEFAULT_FORM);
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
      if (bindingType === 'menus')
        await updateRoleMenus(bindingTarget.role_id, resolvedDeptBindings);
      else
        await updateRoleDataScope(bindingTarget.role_id, {
          data_scope: bindingDataScope,
          dept_check_strictly: bindingStrict,
          dept_ids:
            bindingDataScope === '2'
              ? deptBindingIds(selectedBindings, resolvedDeptBindings, bindingStrict)
              : [],
        });
      toast.success(t('messages.rolePermissionsUpdated'));
      setBindingTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [
    bindingDataScope,
    bindingStrict,
    bindingTarget,
    bindingType,
    resolvedDeptBindings,
    selectedBindings,
    t,
  ]);

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
      <AdminBreadcrumbs heading={t('pages.roleManagement')} action={toolbarAction()} />
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
      {dialogs()}
    </DashboardContent>
  );

  function toolbarAction() {
    return (
      <RoleToolbar
        t={t}
        canAdd={canAdd}
        canDelete={canDelete}
        canExport={canExport}
        selectedCount={selected.length}
        onCreate={openCreate}
        onBatchDelete={() => setBatchDeleteOpen(true)}
        onExport={submitExport}
      />
    );
  }

  function dialogs() {
    return (
      <RoleManagementDialogs
        t={t}
        form={form}
        creating={creating}
        editing={editing}
        submitting={submitting}
        binding={{
          target: bindingTarget,
          type: bindingType,
          nodes: bindingNodes,
          selected: selectedBindings,
          strict: bindingStrict,
          dataScope: bindingDataScope,
          loading: bindingLoading,
          onSelectedChange: setSelectedBindings,
          onStrictChange: setBindingStrict,
          onDataScopeChange: setBindingDataScope,
          onResolvedSelectionChange: setResolvedDeptBindings,
        }}
        usersTarget={usersTarget}
        deleteTarget={deleteTarget}
        batchDeleteOpen={batchDeleteOpen}
        selectedCount={selected.length}
        setForm={setForm}
        onDialogClose={closeDialog}
        onRoleSubmit={submitRole}
        onBindingSubmit={saveBindings}
        onBindingClose={() => setBindingTarget(null)}
        onUsersClose={() => setUsersTarget(null)}
        onBatchDeleteClose={() => setBatchDeleteOpen(false)}
        onBatchDeleteConfirm={confirmBatchDelete}
        onDeleteClose={() => setDeleteTarget(null)}
        onDeleteConfirm={confirmDelete}
      />
    );
  }
}
