import type { Menu } from 'src/entities/menu';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useMenus } from 'src/entities/menu';
import { useHasPermission } from 'src/entities/session';

import { createMenu, deleteMenu, updateMenu, updateMenuSorts } from 'src/features/menu-management';

import { DEFAULT_FORM, DEFAULT_FILTERS } from './constants';
import { toInput, menuHead, flattenMenuRows } from './helpers';

export function useMenuManagementController() {
  const resources = useMenuResources();
  const dialogs = useMenuDialogState();
  const crud = useMenuCrudActions({ dialogs, t: resources.t });
  const sort = useMenuSortAction({ dialogs, t: resources.t });
  const deletion = useMenuDeleteAction({ dialogs, t: resources.t });

  return { resources, dialogs, actions: { ...crud, ...sort, ...deletion } };
}

export type MenuManagementController = ReturnType<typeof useMenuManagementController>;

function useMenuResources() {
  const { t } = useTranslate('admin');
  const filters = useLocalDateTimeFilterState(DEFAULT_FILTERS);
  const menus = useMenus(0, 1000, filters.query);
  const allMenus = useMenus(0, 1000);
  const [expanded, setExpanded] = useState<string[]>([]);
  const head = useMemo(() => menuHead(t), [t]);
  const treeRows = useMemo(() => flattenMenuRows(menus.items, expanded), [expanded, menus.items]);
  const allIds = useMemo(() => allMenus.items.map((menu) => menu.menu_id), [allMenus.items]);
  const canAdd = useHasPermission('system:menu:add');

  return {
    t,
    filters: filters.draft,
    setFilters: filters.change,
    filterError: filters.error,
    menus,
    allMenus,
    expanded,
    setExpanded,
    head,
    treeRows,
    allIds,
    canAdd,
  };
}

function useMenuDialogState() {
  const [form, setForm] = useState(DEFAULT_FORM);
  const [editing, setEditing] = useState<Menu | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Menu | null>(null);
  const [sortEdits, setSortEdits] = useState<Record<string, number>>({});

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
    sortEdits,
    setSortEdits,
  };
}

type MenuCrudOptions = {
  dialogs: ReturnType<typeof useMenuDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
};

function useMenuCrudActions({ dialogs, t }: MenuCrudOptions) {
  const { form, editing, setForm, setEditing, setCreating, setSubmitting, setSortEdits } = dialogs;
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
  const openCreateChild = useCallback(
    (menu: Menu) => {
      setEditing(null);
      setCreating(true);
      setForm({ ...DEFAULT_FORM, parent_id: menu.menu_id });
    },
    [setCreating, setEditing, setForm]
  );
  const openEdit = useCallback(
    (menu: Menu) => {
      setEditing(menu);
      setForm(toInput(menu));
    },
    [setEditing, setForm]
  );
  const submitMenu = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editing) await updateMenu(editing.menu_id, form);
      else await createMenu(form);
      toast.success(t('messages.saved'));
      setSortEdits({});
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeDialog, editing, form, setSortEdits, setSubmitting, t]);

  return { closeDialog, openCreate, openCreateChild, openEdit, submitMenu };
}

type MenuActionOptions = {
  dialogs: ReturnType<typeof useMenuDialogState>;
  t: ReturnType<typeof useTranslate>['t'];
};

function useMenuSortAction({ dialogs, t }: MenuActionOptions) {
  const saveSorts = useCallback(async () => {
    const items = Object.entries(dialogs.sortEdits).map(([id, order_num]) => ({ id, order_num }));
    if (items.length === 0) return;
    dialogs.setSubmitting(true);
    try {
      await updateMenuSorts(items);
      toast.success(t('messages.sortSaved'));
      dialogs.setSortEdits({});
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      dialogs.setSubmitting(false);
    }
  }, [dialogs, t]);

  return { saveSorts };
}

function useMenuDeleteAction({ dialogs, t }: MenuActionOptions) {
  const confirmDelete = useCallback(async () => {
    if (!dialogs.deleteTarget) return;
    try {
      await deleteMenu(dialogs.deleteTarget.menu_id);
      toast.success(t('messages.deleted'));
      dialogs.setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [dialogs, t]);

  return { confirmDelete };
}
