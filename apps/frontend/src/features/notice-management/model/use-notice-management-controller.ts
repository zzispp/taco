'use client';

import type { NoticeInput, NoticeFilters, NoticeSummary } from 'src/entities/notice';

import { useState } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { useHasPermission } from 'src/entities/session';
import { useNotice, useNotices, useNoticeReaders, NOTICE_PERMISSION } from 'src/entities/notice';

import { resetNoticeQuery } from './table-actions';
import { noticeManagementCapabilities } from './permissions';
import { requireNoticeDeleteTarget } from './mutation-preconditions';
import { createNotice, deleteNotice, updateNotice, deleteNotices } from '../api';

const DEFAULT_FILTERS: NoticeFilters = {
  notice_title: '',
  create_by: '',
  notice_type: '',
};

export function useNoticeManagementController() {
  const { t } = useTranslate('admin');
  const state = useNoticeManagementState();
  const permissions = useNoticePermissions();
  const resources = useNoticeResources(state, permissions);
  const mutation = useNoticeMutation();
  const actions = buildNoticeActions({ state, mutation, t });
  return { state, permissions, resources, actions, pending: mutation.pending };
}

export type NoticeManagementController = ReturnType<typeof useNoticeManagementController>;

function useNoticeManagementState() {
  const table = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const [filterDraft, setFilterDraft] = useState<NoticeFilters>(DEFAULT_FILTERS);
  const [filters, setFilters] = useState<NoticeFilters>(DEFAULT_FILTERS);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [detailId, setDetailId] = useState<string | null>(null);
  const [readerTarget, setReaderTarget] = useState<NoticeSummary | null>(null);
  const [readerDraft, setReaderDraft] = useState('');
  const [readerQuery, setReaderQuery] = useState('');
  const readerTable = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    scopeKey: [readerTarget?.notice_id ?? '', readerQuery].join('\u0000'),
  });
  const [deleteTarget, setDeleteTarget] = useState<NoticeSummary | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  return {
    table,
    readerTable,
    filterDraft,
    setFilterDraft,
    filters,
    setFilters,
    creating,
    setCreating,
    editingId,
    setEditingId,
    detailId,
    setDetailId,
    readerTarget,
    setReaderTarget,
    readerDraft,
    setReaderDraft,
    readerQuery,
    setReaderQuery,
    deleteTarget,
    setDeleteTarget,
    batchDeleteOpen,
    setBatchDeleteOpen,
  };
}

function useNoticePermissions() {
  return noticeManagementCapabilities({
    canList: useHasPermission(NOTICE_PERMISSION.LIST),
    canQuery: useHasPermission(NOTICE_PERMISSION.QUERY),
    canAdd: useHasPermission(NOTICE_PERMISSION.ADD),
    canEdit: useHasPermission(NOTICE_PERMISSION.EDIT),
    canRemove: useHasPermission(NOTICE_PERMISSION.REMOVE),
  });
}

type NoticeState = ReturnType<typeof useNoticeManagementState>;
type NoticePermissions = ReturnType<typeof useNoticePermissions>;

function useNoticeResources(state: NoticeState, permissions: NoticePermissions) {
  return {
    notices: useNotices(state.table.cursorRequest, state.filters),
    detail: useNotice(state.detailId, permissions.canOpenDetail),
    editor: useNotice(state.editingId, permissions.canEdit),
    readers: useNoticeReaders({
      noticeId: state.readerTarget?.notice_id ?? null,
      request: state.readerTable.cursorRequest,
      params: { search_value: state.readerQuery },
      enabled: permissions.canViewReaders,
    }),
  };
}

function useNoticeMutation() {
  const [pending, setPending] = useState<string | null>(null);
  const run = async (key: string, action: () => Promise<unknown>, onSuccess: () => void) => {
    setPending(key);
    try {
      await action();
      onSuccess();
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      setPending(null);
    }
  };
  return { pending, run };
}

type ActionOptions = Readonly<{
  state: NoticeState;
  mutation: ReturnType<typeof useNoticeMutation>;
  t: ReturnType<typeof useTranslate>['t'];
}>;

function buildNoticeActions({ state, mutation, t }: ActionOptions) {
  const closeEditor = () => {
    state.setCreating(false);
    state.setEditingId(null);
  };
  const submit = (input: NoticeInput) => {
    const editingId = state.editingId;
    const action = editingId ? () => updateNotice(editingId, input) : () => createNotice(input);
    return mutation.run(editingId ? `edit:${editingId}` : 'create', action, () => {
      state.table.onResetCursor();
      closeEditor();
      toast.success(t('messages.saved'));
    });
  };
  return {
    closeEditor,
    submit,
    ...buildFilterActions(state),
    ...buildReaderActions(state),
    ...buildDeleteActions({ state, mutation, t }),
  };
}

function buildFilterActions(state: NoticeState) {
  const applyFilters = () => {
    state.setFilters(state.filterDraft);
    resetNoticeQuery(state.table);
  };
  const resetFilters = () => {
    state.setFilterDraft(DEFAULT_FILTERS);
    state.setFilters(DEFAULT_FILTERS);
    resetNoticeQuery(state.table);
  };
  return { applyFilters, resetFilters };
}

function buildReaderActions(state: NoticeState) {
  const openReaders = (notice: NoticeSummary) => {
    state.setReaderTarget(notice);
    state.setReaderDraft('');
    state.setReaderQuery('');
    state.readerTable.onResetCursor();
  };
  const searchReaders = () => {
    state.setReaderQuery(state.readerDraft.trim());
    state.readerTable.onResetCursor();
  };
  return { openReaders, searchReaders };
}

type DeleteActionOptions = Readonly<{
  state: NoticeState;
  mutation: ReturnType<typeof useNoticeMutation>;
  t: ReturnType<typeof useTranslate>['t'];
}>;

function buildDeleteActions({ state, mutation, t }: DeleteActionOptions) {
  const confirmDelete = () => {
    const target = requireNoticeDeleteTarget(state.deleteTarget);
    return mutation.run(
      `delete:${target.notice_id}`,
      () => deleteNotice(target.notice_id),
      () => {
        state.setDeleteTarget(null);
        state.table.onResetCursor();
        toast.success(t('messages.deleted'));
      }
    );
  };
  const confirmBatchDelete = () =>
    mutation.run(
      'delete:batch',
      () => deleteNotices(state.table.selected),
      () => {
        state.table.onResetCursor();
        state.setBatchDeleteOpen(false);
        toast.success(t('messages.deleted'));
      }
    );
  return { confirmDelete, confirmBatchDelete };
}
