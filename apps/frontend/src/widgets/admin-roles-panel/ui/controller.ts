import type { Role } from 'src/entities/role';
import type { TreeSelectNode } from 'src/entities/system';
import type { useTranslate } from 'src/shared/i18n/use-locales';

import { useState, useCallback } from 'react';

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

export function useRoleManagementController() {
  const resources = useRoleResources();
  const dialogs = useRoleDialogState();
  const binding = useRoleBindingState();
  const crud = useRoleCrudActions({ dialogs, t: resources.t });
  const bindingActions = useRoleBindingActions({ binding, dialogs, t: resources.t });
  const deletion = useRoleDeletionActions({
    dialogs,
    selectableRoles: resources.selectableRoles,
    t: resources.t,
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

function useRoleDialogState() {
  const [form, setForm] = useState(DEFAULT_FORM);
  const [editing, setEditing] = useState<Role | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Role | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>([]);
  const [usersTarget, setUsersTarget] = useState<Role | null>(null);

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
    usersTarget,
    setUsersTarget,
  };
}

function useRoleBindingState() {
  const [target, setTarget] = useState<Role | null>(null);
  const [type, setType] = useState<'menus' | 'depts'>('menus');
  const [selected, setSelected] = useState<string[]>([]);
  const [resolvedDeptBindings, setResolvedDeptBindings] = useState<string[]>([]);
  const [nodes, setNodes] = useState<TreeSelectNode[]>([]);
  const [strict, setStrict] = useState(true);
  const [dataScope, setDataScope] = useState('5');
  const [loading, setLoading] = useState(false);

  return {
    target,
    setTarget,
    type,
    setType,
    selected,
    setSelected,
    resolvedDeptBindings,
    setResolvedDeptBindings,
    nodes,
    setNodes,
    strict,
    setStrict,
    dataScope,
    setDataScope,
    loading,
    setLoading,
  };
}

type RoleCrudOptions = {
  dialogs: ReturnType<typeof useRoleDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
};

function useRoleCrudActions({ dialogs, t }: RoleCrudOptions) {
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
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, setSubmitting, t]);

  return { closeDialog, openCreate, openEdit, submitRole };
}

type RoleBindingOptions = {
  binding: ReturnType<typeof useRoleBindingState>;
  dialogs: ReturnType<typeof useRoleDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
};

function useRoleBindingActions({ binding, dialogs, t }: RoleBindingOptions) {
  const openBindings = useOpenRoleBindings(binding, t);
  const saveBindings = useSaveRoleBindings({ binding, setSubmitting: dialogs.setSubmitting, t });
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
};

function useSaveRoleBindings({ binding, setSubmitting, t }: SaveRoleBindingsOptions) {
  return useCallback(async () => {
    if (!binding.target) return;
    setSubmitting(true);
    try {
      if (binding.type === 'menus')
        await updateRoleMenus(binding.target.role_id, binding.resolvedDeptBindings);
      else await updateRoleDataScope(binding.target.role_id, roleDataScopePayload(binding));
      toast.success(t('messages.rolePermissionsUpdated'));
      binding.setTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveBindingsFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [binding, setSubmitting, t]);
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
};

function useRoleDeletionActions({ dialogs, selectableRoles, t }: RoleDeletionOptions) {
  const confirmDelete = useConfirmRoleDelete(dialogs, t);
  const confirmBatchDelete = useConfirmRoleBatchDelete(dialogs, t);
  const toggleAll = useCallback(
    (checked: boolean) =>
      dialogs.setSelected(checked ? selectableRoles.map((role) => role.role_id) : []),
    [dialogs, selectableRoles]
  );

  return { confirmDelete, confirmBatchDelete, toggleAll };
}

function useConfirmRoleDelete(
  dialogs: ReturnType<typeof useRoleDialogState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (!dialogs.deleteTarget) return;
    try {
      await deleteRole(dialogs.deleteTarget.role_id);
      toast.success(t('messages.deleted'));
      dialogs.setDeleteTarget(null);
      dialogs.setSelected((current) =>
        current.filter((id) => id !== dialogs.deleteTarget?.role_id)
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [dialogs, t]);
}

function useConfirmRoleBatchDelete(
  dialogs: ReturnType<typeof useRoleDialogState>,
  t: ReturnType<typeof useTranslate>['t']
) {
  return useCallback(async () => {
    if (dialogs.selected.length === 0) return;
    try {
      await deleteRoles(dialogs.selected);
      toast.success(t('messages.deleted'));
      dialogs.setSelected([]);
      dialogs.setBatchDeleteOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [dialogs, t]);
}
