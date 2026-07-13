import type { SystemUser } from 'src/entities/user';
import type { useTranslate } from 'src/shared/i18n/use-locales';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import {
  createUser,
  deleteUser,
  updateUser,
  exportUsers,
  importUsers,
  deleteUsers,
  getUserRoles,
  updateUserRoles,
  resetUserPassword,
  downloadUserImportTemplate,
} from 'src/features/user-management';

import { toInput } from './helpers';
import { DEFAULT_FORM } from './constants';
import { useUserResources } from './resources';

export function useUserManagementController() {
  const state = useUserState();
  const resources = useUserResources();
  const crud = useUserCrudActions({ state, resources });
  const roles = useUserRoleActions({ state, t: resources.t });
  const password = useUserPasswordAction({ state, t: resources.t });
  const imports = useUserImportActions({ state, t: resources.t });
  const deletion = useUserDeleteActions({ state, t: resources.t });
  const table = useUserTableActions({ state, resources });

  return {
    resources,
    state,
    actions: { ...crud, ...roles, ...password, ...imports, ...deletion, ...table },
  };
}

export type UserManagementController = ReturnType<typeof useUserManagementController>;

function useUserState() {
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

  return {
    form,
    setForm,
    editing,
    setEditing,
    creating,
    setCreating,
    submitting,
    setSubmitting,
    deleteTarget,
    setDeleteTarget,
    batchDeleteOpen,
    setBatchDeleteOpen,
    selected,
    setSelected,
    passwordTarget,
    setPasswordTarget,
    newPassword,
    setNewPassword,
    roleTarget,
    setRoleTarget,
    assignedRoles,
    setAssignedRoles,
    importOpen,
    setImportOpen,
    importFile,
    setImportFile,
    updateSupport,
    setUpdateSupport,
  };
}

type UserActionOptions = {
  state: ReturnType<typeof useUserState>;
  resources?: ReturnType<typeof useUserResources>;
  t?: ReturnType<typeof useTranslate>['t'];
};

function useUserCrudActions({
  state,
  resources,
}: Required<Pick<UserActionOptions, 'state' | 'resources'>>) {
  const closeDialog = useCallback(() => {
    state.setEditing(null);
    state.setCreating(false);
    state.setForm(DEFAULT_FORM);
  }, [state]);
  const openCreate = useCallback(() => {
    state.setEditing(null);
    state.setCreating(true);
    state.setForm({
      ...DEFAULT_FORM,
      role_ids: [resources.roles[0]?.role_id ?? ''].filter(Boolean),
    });
  }, [resources.roles, state]);
  const openEdit = useCallback(
    (user: SystemUser) => {
      state.setEditing(user);
      state.setForm(toInput(user));
    },
    [state]
  );
  const submitUser = useSubmitUser({ state, closeDialog, t: resources.t });

  return { closeDialog, openCreate, openEdit, submitUser };
}

type SubmitUserOptions = {
  state: ReturnType<typeof useUserState>;
  closeDialog: () => void;
  t: ReturnType<typeof useTranslate>['t'];
};

function useSubmitUser({ state, closeDialog, t }: SubmitUserOptions) {
  return useCallback(async () => {
    state.setSubmitting(true);
    try {
      if (state.editing) await updateUser(state.editing.user_id, state.form);
      else await createUser(state.form);
      toast.success(t('messages.saved'));
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeDialog, state, t]);
}

function useUserRoleActions({ state, t }: Required<Pick<UserActionOptions, 'state' | 't'>>) {
  const openRoles = useCallback(
    async (user: SystemUser) => {
      state.setRoleTarget(user);
      try {
        const payload = await getUserRoles(user.user_id);
        state.setAssignedRoles(payload.role_ids);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      }
    },
    [state, t]
  );
  const submitRoles = useCallback(async () => {
    if (!state.roleTarget) return;
    state.setSubmitting(true);
    try {
      await updateUserRoles(state.roleTarget.user_id, state.assignedRoles);
      toast.success(t('messages.rolePermissionsUpdated'));
      state.setRoleTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [state, t]);

  return { openRoles, submitRoles };
}

function useUserPasswordAction({ state, t }: Required<Pick<UserActionOptions, 'state' | 't'>>) {
  const submitPassword = useCallback(async () => {
    if (!state.passwordTarget) return;
    state.setSubmitting(true);
    try {
      await resetUserPassword(state.passwordTarget.user_id, state.newPassword);
      toast.success(t('messages.saved'));
      state.setPasswordTarget(null);
      state.setNewPassword('');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [state, t]);

  return { submitPassword };
}

function useUserImportActions({ state, t }: Required<Pick<UserActionOptions, 'state' | 't'>>) {
  const closeImportDialog = useCallback(() => {
    state.setImportOpen(false);
    state.setImportFile(null);
    state.setUpdateSupport(false);
  }, [state]);
  const submitImport = useCallback(async () => {
    if (!state.importFile) return;
    state.setSubmitting(true);
    try {
      const result = await importUsers(state.importFile, state.updateSupport);
      toast.success(result.message || t('messages.importSuccess'));
      closeImportDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.importFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeImportDialog, state, t]);

  return { closeImportDialog, submitImport, downloadUserImportTemplate };
}

function useUserDeleteActions({ state, t }: Required<Pick<UserActionOptions, 'state' | 't'>>) {
  const confirmDelete = useCallback(async () => {
    if (!state.deleteTarget) return;
    try {
      await deleteUser(state.deleteTarget.user_id);
      toast.success(t('messages.deleted'));
      state.setDeleteTarget(null);
      state.setSelected((current) => current.filter((id) => id !== state.deleteTarget?.user_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);
  const confirmBatchDelete = useCallback(async () => {
    if (state.selected.length === 0) return;
    try {
      await deleteUsers(state.selected);
      toast.success(t('messages.deleted'));
      state.setSelected([]);
      state.setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);

  return { confirmDelete, confirmBatchDelete };
}

function useUserTableActions({
  state,
  resources,
}: Required<Pick<UserActionOptions, 'state' | 'resources'>>) {
  const selectDept = useCallback(
    (dept_id: string) => {
      resources.setFilters({ ...resources.filters, dept_id });
    },
    [resources]
  );
  const toggleAll = useCallback(
    (checked: boolean) => {
      state.setSelected(checked ? resources.selectableUsers.map((user) => user.user_id) : []);
    },
    [resources.selectableUsers, state]
  );
  const openPassword = useCallback(
    (user: SystemUser) => {
      state.setPasswordTarget(user);
      state.setNewPassword('');
    },
    [state]
  );
  const submitExport = useCallback(async () => {
    if (resources.filterError) {
      toast.error(resources.t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[resources.filterError]));
      return;
    }
    try {
      await exportUsers(resources.filterQuery);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : resources.t('messages.exportFailed'));
    }
  }, [resources]);

  return { selectDept, toggleAll, openPassword, submitExport };
}
