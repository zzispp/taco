import type { Dept, TreeSelectNode } from 'src/entities/system';

import { useMemo, useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useDepts } from 'src/entities/system';
import { useHasPermission } from 'src/entities/session';

import { getDeptTree, getDeptExclude, systemMutations } from 'src/features/system-management';

import { DEFAULT_INPUT, DEFAULT_FILTERS } from './dept-constants';
import { toInput, deptHead, flattenDeptRows } from './dept-helpers';

export function useDeptManagementController() {
  const state = useDeptState();
  const resources = useDeptResources(state.filterQuery, state.expanded);
  const parent = useDeptParentLoader({ state, t: resources.t });
  const crud = useDeptCrudActions({
    state,
    loadParentNodes: parent.loadParentNodes,
    t: resources.t,
  });
  const sort = useDeptSortAction({ state, t: resources.t });
  const deletion = useDeptDeleteAction({ state, t: resources.t });

  return { resources, state, actions: { ...parent, ...crud, ...sort, ...deletion } };
}

export type DeptManagementController = ReturnType<typeof useDeptManagementController>;

function useDeptState() {
  const filters = useLocalDateTimeFilterState(DEFAULT_FILTERS);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [form, setForm] = useState(DEFAULT_INPUT);
  const [editing, setEditing] = useState<Dept | null>(null);
  const [creating, setCreating] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Dept | null>(null);
  const [sortEdits, setSortEdits] = useState<Record<string, number>>({});
  const [parentNodes, setParentNodes] = useState<TreeSelectNode[]>([]);

  return {
    filters: filters.draft,
    setFilters: filters.change,
    filterQuery: filters.query,
    filterError: filters.error,
    expanded,
    setExpanded,
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
    parentNodes,
    setParentNodes,
  };
}

function useDeptResources(filters: Readonly<Record<string, string>>, expanded: string[]) {
  const { t } = useTranslate('admin');
  const resource = useDepts(0, 1000, filters);
  const head = useMemo(() => deptHead(t), [t]);
  const rows = useMemo(() => flattenDeptRows(resource.items, expanded), [expanded, resource.items]);
  const allIds = useMemo(() => resource.items.map((dept) => dept.dept_id), [resource.items]);
  const canAdd = useHasPermission('system:dept:add');

  return { t, resource, head, rows, allIds, canAdd };
}

type DeptActionOptions = {
  state: ReturnType<typeof useDeptState>;
  t: ReturnType<typeof useTranslate>['t'];
};

type DeptParentOptions = DeptActionOptions;

function useDeptParentLoader({ state, t }: DeptParentOptions) {
  const loadParentNodes = useCallback(
    async (loader: () => Promise<TreeSelectNode[]>) => {
      try {
        state.setParentNodes(await loader());
      } catch (error) {
        toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
      }
    },
    [state, t]
  );

  return { loadParentNodes };
}

type DeptCrudOptions = DeptActionOptions & {
  loadParentNodes: (loader: () => Promise<TreeSelectNode[]>) => Promise<void>;
};

function useDeptCrudActions({ state, loadParentNodes, t }: DeptCrudOptions) {
  const closeDialog = useCallback(() => {
    state.setEditing(null);
    state.setCreating(false);
    state.setForm(DEFAULT_INPUT);
  }, [state]);
  const openCreate = useCallback(async () => {
    state.setEditing(null);
    state.setCreating(true);
    state.setForm(DEFAULT_INPUT);
    await loadParentNodes(() => getDeptTree());
  }, [loadParentNodes, state]);
  const openCreateChild = useCallback(
    async (dept: Dept) => {
      state.setEditing(null);
      state.setCreating(true);
      state.setForm({ ...DEFAULT_INPUT, parent_id: dept.dept_id });
      await loadParentNodes(() => getDeptTree());
    },
    [loadParentNodes, state]
  );
  const openEdit = useCallback(
    async (dept: Dept) => {
      state.setEditing(dept);
      state.setForm(toInput(dept));
      await loadParentNodes(() => getDeptExclude(dept.dept_id));
    },
    [loadParentNodes, state]
  );
  const submitDept = useSubmitDept({ state, closeDialog, t });

  return { closeDialog, openCreate, openCreateChild, openEdit, submitDept };
}

type SubmitDeptOptions = DeptActionOptions & { closeDialog: () => void };

function useSubmitDept({ state, closeDialog, t }: SubmitDeptOptions) {
  return useCallback(async () => {
    state.setSubmitting(true);
    try {
      if (state.editing) await systemMutations.updateDept(state.editing.dept_id, state.form);
      else await systemMutations.createDept(state.form);
      toast.success(t('messages.saved'));
      state.setSortEdits({});
      closeDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [closeDialog, state, t]);
}

function useDeptSortAction({ state, t }: DeptActionOptions) {
  const saveSorts = useCallback(async () => {
    const items = Object.entries(state.sortEdits).map(([id, order_num]) => ({ id, order_num }));
    if (items.length === 0) return;
    state.setSubmitting(true);
    try {
      await systemMutations.updateDeptSorts(items);
      toast.success(t('messages.sortSaved'));
      state.setSortEdits({});
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      state.setSubmitting(false);
    }
  }, [state, t]);

  return { saveSorts };
}

function useDeptDeleteAction({ state, t }: DeptActionOptions) {
  const confirmDelete = useCallback(async () => {
    if (!state.deleteTarget) return;
    try {
      await systemMutations.deleteDept(state.deleteTarget.dept_id);
      toast.success(t('messages.deleted'));
      state.setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [state, t]);

  return { confirmDelete };
}
