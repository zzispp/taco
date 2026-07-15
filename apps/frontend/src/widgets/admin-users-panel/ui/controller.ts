import type { SystemUser } from 'src/entities/user';
import type { PasswordPolicy } from 'src/entities/system';
import type { useTranslate } from 'src/shared/i18n/use-locales';

import { useState, useEffect, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import {
  createUser,
  deleteUser,
  updateUser,
  exportUsers,
  importUsers,
  deleteUsers,
  downloadUserImportTemplate,
} from 'src/features/user-management';

import { toInput } from './helpers';
import { DEFAULT_FORM } from './constants';
import { useUserResources } from './resources';
import { useUserRoleActions, useUserPasswordAction } from '../model/use-user-security-actions';
import { validateAdminUserForm, adminUserValidationMessages } from '../model/user-form-validation';

export function useUserManagementController() {
  const state = useUserState();
  const resources = useUserResources();
  const clearSelected = state.setSelected;
  useEffect(
    () => clearSelected([]),
    [clearSelected, resources.filterQuery, resources.table.cursor, resources.table.limit]
  );
  const resetList = resources.table.onResetCursor;
  const crud = useUserCrudActions({
    state,
    t: resources.t,
    passwordPolicy: resources.passwordPolicy,
    resetList,
  });
  const roles = useUserRoleActions({ state, t: resources.t, resetList });
  const password = useUserPasswordAction({
    state,
    t: resources.t,
    passwordPolicy: resources.passwordPolicy,
    resetList,
  });
  const imports = useUserImportActions({ state, t: resources.t, resetList });
  const deletion = useUserDeleteActions({ state, t: resources.t, resetList });
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
  passwordPolicy?: PasswordPolicy;
  resetList?: () => void;
};

type ValidatedUserActionOptions = Required<Pick<UserActionOptions, 'state' | 't'>> & {
  passwordPolicy: PasswordPolicy | undefined;
  resetList: () => void;
};

function useUserCrudActions({ state, t, passwordPolicy, resetList }: ValidatedUserActionOptions) {
  const closeDialog = useCallback(() => {
    state.setEditing(null);
    state.setCreating(false);
    state.setForm(DEFAULT_FORM);
  }, [state]);
  const openCreate = useCallback(() => {
    state.setEditing(null);
    state.setCreating(true);
    state.setForm(DEFAULT_FORM);
  }, [state]);
  const openEdit = useCallback(
    (user: SystemUser) => {
      state.setEditing(user);
      state.setForm(toInput(user));
    },
    [state]
  );
  const submitUser = useSubmitUser({ state, closeDialog, t, passwordPolicy, resetList });

  return { closeDialog, openCreate, openEdit, submitUser };
}

type SubmitUserOptions = {
  state: ReturnType<typeof useUserState>;
  closeDialog: () => void;
  t: ReturnType<typeof useTranslate>['t'];
  passwordPolicy: PasswordPolicy | undefined;
  resetList: () => void;
};

function useSubmitUser({ state, closeDialog, t, passwordPolicy, resetList }: SubmitUserOptions) {
  return useCallback(async () => {
    const validationError = validateAdminUserForm(state.form, {
      mode: state.editing ? 'edit' : 'create',
      policy: passwordPolicy,
      messages: adminUserValidationMessages(t),
    });
    if (validationError) {
      toast.error(validationError);
      return;
    }
    state.setSubmitting(true);
    try {
      if (state.editing) await updateUser(state.editing.user_id, state.form);
      else await createUser(state.form);
      toast.success(t('messages.saved'));
      resetList();
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeDialog, passwordPolicy, resetList, state, t]);
}

function useUserImportActions({
  state,
  t,
  resetList,
}: Required<Pick<UserActionOptions, 'state' | 't' | 'resetList'>>) {
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
      resetList();
      closeImportDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.importFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeImportDialog, resetList, state, t]);

  return { closeImportDialog, submitImport, downloadUserImportTemplate };
}

function useUserDeleteActions({
  state,
  t,
  resetList,
}: Required<Pick<UserActionOptions, 'state' | 't' | 'resetList'>>) {
  const confirmDelete = useCallback(async () => {
    if (!state.deleteTarget) return;
    try {
      await deleteUser(state.deleteTarget.user_id);
      toast.success(t('messages.deleted'));
      resetList();
      state.setDeleteTarget(null);
      state.setSelected((current) => current.filter((id) => id !== state.deleteTarget?.user_id));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [resetList, state, t]);
  const confirmBatchDelete = useCallback(async () => {
    if (state.selected.length === 0) return;
    try {
      await deleteUsers(state.selected);
      toast.success(t('messages.deleted'));
      resetList();
      state.setSelected([]);
      state.setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [resetList, state, t]);

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
