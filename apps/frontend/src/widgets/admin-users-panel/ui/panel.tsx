'use client';

import type { SystemUser } from 'src/entities/user';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { withSelectionHead } from 'src/shared/ui/admin';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import { useUsers, useUserFormOptions } from 'src/entities/user';

import {
  createUser,
  deleteUser,
  updateUser,
  exportUsers,
  importUsers,
  deleteUsers,
  getUserRoles,
  updateUserRoles,
  updateUserStatus,
  resetUserPassword,
  downloadUserImportTemplate,
} from 'src/features/user-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { UserToolbar } from './toolbar';
import { UserTableSection } from './table-section';
import { UserConfirmDialogs } from './confirm-dialogs';
import { DEFAULT_FORM, DEFAULT_FILTERS } from './constants';
import { UserManagementDialogs } from './management-dialogs';
import { toggle, toInput, userHead, showError, flattenDeptNames } from './helpers';

export function UserManagementView() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const users = useUsers(table.page, table.rowsPerPage, filters);
  const options = useUserFormOptions();
  const roles = useMemo(() => options.data?.roles ?? [], [options.data?.roles]);
  const posts = useMemo(() => options.data?.posts ?? [], [options.data?.posts]);
  const deptTree = useMemo(() => options.data?.depts ?? [], [options.data?.depts]);
  const depts = useMemo(() => flattenDeptNames(deptTree), [deptTree]);
  const [form, setForm] = useState(DEFAULT_FORM);
  const [editing, setEditing] = useState<SystemUser | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<SystemUser | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [passwordTarget, setPasswordTarget] = useState<SystemUser | null>(null);
  const [newPassword, setNewPassword] = useState('');
  const [roleTarget, setRoleTarget] = useState<SystemUser | null>(null);
  const [assignedRoles, setAssignedRoles] = useState<string[]>([]);
  const [importOpen, setImportOpen] = useState(false);
  const [importFile, setImportFile] = useState<File | null>(null);
  const [updateSupport, setUpdateSupport] = useState(false);
  const head = useMemo(() => userHead(t), [t]);
  const canAdd = useHasPermission('system:user:add');
  const canDelete = useHasPermission('system:user:remove');
  const canImport = useHasPermission('system:user:import');
  const canExport = useHasPermission('system:user:export');
  const loadingHead = useMemo(
    () => (canDelete ? withSelectionHead(head) : head),
    [canDelete, head]
  );
  const selectableUsers = useMemo(() => users.items.filter((user) => !user.system), [users.items]);

  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, []);

  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm({ ...DEFAULT_FORM, role_ids: [roles[0]?.role_id ?? ''].filter(Boolean) });
  }, [roles]);

  const openEdit = useCallback((user: SystemUser) => {
    setEditing(user);
    setForm(toInput(user));
  }, []);

  const submitUser = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateUser(editing.user_id, form);
      else await createUser(form);
      toast.success(t('messages.saved'));
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
      await deleteUser(deleteTarget.user_id);
      toast.success(t('messages.deleted'));
      setDeleteTarget(null);
      setSelected((current) => current.filter((id) => id !== deleteTarget.user_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteTarget, t]);

  const confirmBatchDelete = useCallback(async () => {
    if (selected.length === 0) return;
    try {
      await deleteUsers(selected);
      toast.success(t('messages.deleted'));
      setSelected([]);
      setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [selected, t]);

  const submitPassword = useCallback(async () => {
    if (!passwordTarget) return;
    setSubmitting(true);
    try {
      await resetUserPassword(passwordTarget.user_id, newPassword);
      toast.success(t('messages.saved'));
      setPasswordTarget(null);
      setNewPassword('');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [newPassword, passwordTarget, t]);

  const openRoles = useCallback(
    async (user: SystemUser) => {
      setRoleTarget(user);
      try {
        const payload = await getUserRoles(user.user_id);
        setAssignedRoles(payload.role_ids);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      }
    },
    [t]
  );

  const submitRoles = useCallback(async () => {
    if (!roleTarget) return;
    setSubmitting(true);
    try {
      await updateUserRoles(roleTarget.user_id, assignedRoles);
      toast.success(t('messages.rolePermissionsUpdated'));
      setRoleTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [assignedRoles, roleTarget, t]);

  const submitImport = useCallback(async () => {
    if (!importFile) return;
    setSubmitting(true);
    try {
      const result = await importUsers(importFile, updateSupport);
      toast.success(result.message || t('messages.importSuccess'));
      closeImportDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.importFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [importFile, t, updateSupport]);

  const submitExport = useCallback(async () => {
    try {
      await exportUsers(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  }, [filters, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={t('pages.userManagement')} action={toolbar()} />
      <UserTableSection
        table={table}
        filters={filters}
        users={users}
        roles={roles}
        depts={depts}
        posts={posts}
        deptTree={deptTree}
        head={head}
        loadingHead={loadingHead}
        selectableUsers={selectableUsers}
        selected={selected}
        canDelete={canDelete}
        onFilterChange={setFilters}
        onDeptSelect={selectDept}
        onToggleAll={toggleAll}
        onToggleSelected={(id) => setSelected(toggle(selected, id))}
        onEdit={openEdit}
        onDelete={setDeleteTarget}
        onRoles={openRoles}
        onResetPassword={(user) => {
          setPasswordTarget(user);
          setNewPassword('');
        }}
        onStatusChange={(user, status) =>
          updateUserStatus(user.user_id, status).catch(showError(t))
        }
      />
      <UserManagementDialogs
        form={form}
        roles={roles}
        posts={posts}
        deptTree={deptTree}
        editing={editing}
        creating={creating}
        submitting={submitting}
        roleTarget={roleTarget}
        assignedRoles={assignedRoles}
        passwordTarget={passwordTarget}
        newPassword={newPassword}
        importOpen={importOpen}
        importFile={importFile}
        updateSupport={updateSupport}
        setForm={setForm}
        onDialogClose={closeDialog}
        onUserSubmit={submitUser}
        onAssignedRolesChange={setAssignedRoles}
        onRoleClose={() => setRoleTarget(null)}
        onRolesSubmit={submitRoles}
        onPasswordChange={setNewPassword}
        onPasswordClose={() => setPasswordTarget(null)}
        onPasswordSubmit={submitPassword}
        onImportFileChange={setImportFile}
        onUpdateSupportChange={setUpdateSupport}
        onImportTemplate={downloadUserImportTemplate}
        onImportClose={closeImportDialog}
        onImportSubmit={submitImport}
      />
      <UserConfirmDialogs
        t={t}
        deleteTarget={deleteTarget}
        batchDeleteOpen={batchDeleteOpen}
        selectedCount={selected.length}
        onBatchDeleteClose={() => setBatchDeleteOpen(false)}
        onDeleteClose={() => setDeleteTarget(null)}
        onBatchDeleteConfirm={confirmBatchDelete}
        onDeleteConfirm={confirmDelete}
      />
    </DashboardContent>
  );

  function toolbar() {
    return (
      <UserToolbar
        t={t}
        canAdd={canAdd}
        canDelete={canDelete}
        canImport={canImport}
        canExport={canExport}
        selectedCount={selected.length}
        onCreate={openCreate}
        onImport={() => setImportOpen(true)}
        onExport={submitExport}
        onBatchDelete={() => setBatchDeleteOpen(true)}
      />
    );
  }

  function selectDept(dept_id: string) {
    setFilters((current) => ({ ...current, dept_id }));
  }

  function toggleAll(checked: boolean) {
    setSelected(checked ? selectableUsers.map((user) => user.user_id) : []);
  }

  function closeImportDialog() {
    setImportOpen(false);
    setImportFile(null);
    setUpdateSupport(false);
  }
}
