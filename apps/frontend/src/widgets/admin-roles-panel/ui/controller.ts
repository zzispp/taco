import type { Role } from 'src/entities/role';
import type { useTranslate } from 'src/shared/i18n/use-locales';

import { useEffect, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import {
  createRole,
  deleteRole,
  updateRole,
  deleteRoles,
  getRoleDeptTree,
  getRoleMenuTree,
  updateRoleMenus,
  updateRoleDataScope,
} from 'src/features/role-management';

import { DEFAULT_FORM } from './constants';
import { toInput, deptBindingIds } from './helpers';
import { useRoleResources, useRoleExportAction } from './resources';
import { useRoleDialogState, useRoleBindingState } from './controller-state';

export function useRoleManagementController() {
  const resources = useRoleResources();
  const dialogs = useRoleDialogState();
  const clearSelected = dialogs.setSelected;
  useEffect(
    () => clearSelected([]),
    [clearSelected, resources.filterQuery, resources.table.cursor, resources.table.limit]
  );
  const binding = useRoleBindingState();
  const resetList = resources.table.onResetCursor;
  const crud = useRoleCrudActions({ dialogs, t: resources.t, resetList });
  const bindingActions = useRoleBindingActions({
    binding,
    dialogs,
    t: resources.t,
    resetList,
  });
  const deletion = useRoleDeletionActions({
    dialogs,
    selectableRoles: resources.selectableRoles,
    t: resources.t,
    resetList,
  });
  const exportAction = useRoleExportAction({
    filters: resources.filterQuery,
    filterError: resources.filterError,
    t: resources.t,
  });

  return {
    resources,
    dialogs,
    binding,
    actions: { ...crud, ...bindingActions, ...deletion, ...exportAction },
  };
}

export type RoleManagementController = ReturnType<typeof useRoleManagementController>;

type RoleCrudOptions = {
  dialogs: ReturnType<typeof useRoleDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
  resetList: () => void;
};

function useRoleCrudActions({ dialogs, t, resetList }: RoleCrudOptions) {
  const { form, editing, setForm, setEditing, setCreating, setSubmitting } = dialogs;
  const closeDialog = useCallback(() => {
    setEditing(null);
    setCreating(false);
    setForm(DEFAULT_FORM);
  }, [setCreating, setEditing, setForm]);
  const openCreate = useCallback(() => {
    setEditing(null);
    setCreating(true);
    setForm(DEFAULT_FORM);
  }, [setCreating, setEditing, setForm]);
  const openEdit = useCallback(
    (role: Role) => {
      setEditing(role);
      setForm(toInput(role));
    },
    [setEditing, setForm]
  );
  const submitRole = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateRole(editing.role_id, form);
      else await createRole(form);
      toast.success(t('messages.saved'));
      resetList();
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, resetList, setSubmitting, t]);

  return { closeDialog, openCreate, openEdit, submitRole };
}

type RoleBindingOptions = {
  binding: ReturnType<typeof useRoleBindingState>;
  dialogs: ReturnType<typeof useRoleDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
  resetList: () => void;
};

function useRoleBindingActions({ binding, dialogs, t, resetList }: RoleBindingOptions) {
  const openBindings = useOpenRoleBindings(binding, t);
  const saveBindings = useSaveRoleBindings({
    binding,
    setSubmitting: dialogs.setSubmitting,
    t,
    resetList,
  });
  return { openBindings, saveBindings };
}

function useOpenRoleBindings(
  binding: ReturnType<typeof useRoleBindingState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(
    async (role: Role, type: 'menus' | 'depts') => {
      binding.setTarget(role);
      binding.setType(type);
      binding.setResolvedDeptBindings([]);
      binding.setLoading(true);
      try {
        if (type === 'menus') await loadMenuBindings(role, binding);
        else await loadDeptBindings(role, binding);
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.loadBindingsFailed'));
      } finally {
        binding.setLoading(false);
      }
    },
    [binding, t]
  );
}

async function loadMenuBindings(role: Role, binding: ReturnType<typeof useRoleBindingState>) {
  const data = await getRoleMenuTree(role.role_id);
  binding.setNodes(data.menus);
  binding.setSelected(data.checked_keys);
  binding.setStrict(role.menu_check_strictly);
}

async function loadDeptBindings(role: Role, binding: ReturnType<typeof useRoleBindingState>) {
  const data = await getRoleDeptTree(role.role_id);
  binding.setNodes(data.depts);
  binding.setSelected(data.checked_keys);
  binding.setStrict(role.dept_check_strictly);
  binding.setDataScope(role.data_scope);
}

type SaveRoleBindingsOptions = {
  binding: ReturnType<typeof useRoleBindingState>;
  setSubmitting: (submitting: boolean) => void;
  t: ReturnType<typeof useTranslate>['t'];
  resetList: () => void;
};

function useSaveRoleBindings({ binding, setSubmitting, t, resetList }: SaveRoleBindingsOptions) {
  return useCallback(async () => {
    if (!binding.target) return;
    setSubmitting(true);
    try {
      if (binding.type === 'menus')
        await updateRoleMenus(binding.target.role_id, binding.resolvedDeptBindings);
      else await updateRoleDataScope(binding.target.role_id, roleDataScopePayload(binding));
      toast.success(t('messages.rolePermissionsUpdated'));
      resetList();
      binding.setTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [binding, resetList, setSubmitting, t]);
}

function roleDataScopePayload(binding: ReturnType<typeof useRoleBindingState>) {
  return {
    data_scope: binding.dataScope,
    dept_check_strictly: binding.strict,
    dept_ids:
      binding.dataScope === '2'
        ? deptBindingIds(binding.selected, binding.resolvedDeptBindings, binding.strict)
        : [],
  };
}

type RoleDeletionOptions = {
  dialogs: ReturnType<typeof useRoleDialogState>;
  selectableRoles: Role[];
  t: ReturnType<typeof useTranslate>['t'];
  resetList: () => void;
};

function useRoleDeletionActions({ dialogs, selectableRoles, t, resetList }: RoleDeletionOptions) {
  const confirmDelete = useConfirmRoleDelete(dialogs, t, resetList);
  const confirmBatchDelete = useConfirmRoleBatchDelete(dialogs, t, resetList);
  const toggleAll = useCallback(
    (checked: boolean) =>
      dialogs.setSelected(checked ? selectableRoles.map((role) => role.role_id) : []),
    [dialogs, selectableRoles]
  );

  return { confirmDelete, confirmBatchDelete, toggleAll };
}

function useConfirmRoleDelete(
  dialogs: ReturnType<typeof useRoleDialogState>,
  t: ReturnType<typeof useTranslate>['t'],
  resetList: () => void
) {
  return useCallback(async () => {
    if (!dialogs.deleteTarget) return;
    try {
      await deleteRole(dialogs.deleteTarget.role_id);
      toast.success(t('messages.deleted'));
      resetList();
      dialogs.setDeleteTarget(null);
      dialogs.setSelected((current) =>
        current.filter((id) => id !== dialogs.deleteTarget?.role_id)
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [dialogs, resetList, t]);
}

function useConfirmRoleBatchDelete(
  dialogs: ReturnType<typeof useRoleDialogState>,
  t: ReturnType<typeof useTranslate>['t'],
  resetList: () => void
) {
  return useCallback(async () => {
    if (dialogs.selected.length === 0) return;
    try {
      await deleteRoles(dialogs.selected);
      toast.success(t('messages.deleted'));
      resetList();
      dialogs.setSelected([]);
      dialogs.setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [dialogs, resetList, t]);
}
