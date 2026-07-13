'use client';

import type { NoticeInput, NoticeFilters, NoticeSummary } from 'src/entities/notice';

import { useState } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { useHasPermission } from 'src/entities/session';
import { useNotice, useNotices, useNoticeReaders, NOTICE_PERMISSION } from 'src/entities/notice';

import { noticeManagementCapabilities } from './permissions';
import { createNotice, deleteNotice, updateNotice, deleteNotices } from '../api';
import {
  changeNoticePage,
  resetNoticeQuery,
  changeNoticeRowsPerPage,
  updatePageAfterNoticeDelete,
  updatePageAfterNoticeBatchDelete,
} from './table-actions';

const DEFAULT_FILTERS: NoticeFilters = {
  notice_title: '',
  create_by: '',
  notice_type: '',
};
const DEFAULT_PAGE_SIZE = 10;

export function useNoticeManagementController() {
  const { t } = useTranslate('admin');
  const state = useNoticeManagementState();
  const permissions = useNoticePermissions();
  const resources = useNoticeResources(state, permissions);
  const mutation = useNoticeMutation();
  const actions = buildNoticeActions({ state, mutation, t, resources });
  return { state, permissions, resources, actions, pending: mutation.pending };
}

export type NoticeManagementController = ReturnType<typeof useNoticeManagementController>;

function useNoticeManagementState() {
  const noticeTable = useNoticeTable();
  const [filterDraft, setFilterDraft] = useState<NoticeFilters>(DEFAULT_FILTERS);
  const [filters, setFilters] = useState<NoticeFilters>(DEFAULT_FILTERS);
  const [creating, setCreating] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [detailId, setDetailId] = useState<string | null>(null);
  const [readerTarget, setReaderTarget] = useState<NoticeSummary | null>(null);
  const [readerDraft, setReaderDraft] = useState('');
  const [readerQuery, setReaderQuery] = useState('');
  const [readerPage, setReaderPage] = useState(0);
  const [readerRowsPerPage, setReaderRowsPerPage] = useState(DEFAULT_PAGE_SIZE);
  const [deleteTarget, setDeleteTarget] = useState<NoticeSummary | null>(null);
  const [batchDeleteOpen, setBatchDeleteOpen] = useState(false);
  return {
    ...noticeTable,
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
    readerPage,
    setReaderPage,
    readerRowsPerPage,
    setReaderRowsPerPage,
    deleteTarget,
    setDeleteTarget,
    batchDeleteOpen,
    setBatchDeleteOpen,
  };
}

function useNoticeTable() {
  const table = useTable({ defaultRowsPerPage: DEFAULT_PAGE_SIZE });
  return {
    table,
    onPageChange: (event: unknown, page: number) => changeNoticePage({ table, event, page }),
    onRowsPerPageChange: (event: React.ChangeEvent<HTMLInputElement>) =>
      changeNoticeRowsPerPage({ table, event }),
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
    notices: useNotices(state.table.page, state.table.rowsPerPage, state.filters),
    detail: useNotice(state.detailId, permissions.canOpenDetail),
    editor: useNotice(state.editingId, permissions.canEdit),
    readers: useNoticeReaders({
      noticeId: state.readerTarget?.notice_id ?? null,
      page: state.readerPage,
      pageSize: state.readerRowsPerPage,
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
  resources: NoticeResources;
}>;
type NoticeResources = ReturnType<typeof useNoticeResources>;

function buildNoticeActions({ state, mutation, t, resources }: ActionOptions) {
  const closeEditor = () => {
    state.setCreating(false);
    state.setEditingId(null);
  };
  const submit = (input: NoticeInput) => {
    const editingId = state.editingId;
    const action = editingId ? () => updateNotice(editingId, input) : () => createNotice(input);
    return mutation.run(editingId ? `edit:${editingId}` : 'create', action, () => {
      closeEditor();
      toast.success(t('messages.saved'));
    });
  };
  return {
    closeEditor,
    submit,
    ...buildFilterActions(state),
    ...buildReaderActions(state),
    ...buildDeleteActions({ state, mutation, t, resource: resources.notices }),
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
    state.setReaderPage(0);
  };
  const searchReaders = () => {
    state.setReaderQuery(state.readerDraft.trim());
    state.setReaderPage(0);
  };
  return { openReaders, searchReaders };
}

type DeleteActionOptions = Readonly<{
  state: NoticeState;
  mutation: ReturnType<typeof useNoticeMutation>;
  t: ReturnType<typeof useTranslate>['t'];
  resource: NoticeResources['notices'];
}>;

function buildDeleteActions({ state, mutation, t, resource }: DeleteActionOptions) {
  const confirmDelete = () => {
    const target = state.deleteTarget;
    if (!target) return Promise.resolve();
    return mutation.run(
      `delete:${target.notice_id}`,
      () => deleteNotice(target.notice_id),
      () => {
        state.setDeleteTarget(null);
        state.table.setSelected((current) => current.filter((id) => id !== target.notice_id));
        updatePageAfterNoticeDelete(state.table, resource.items.length);
        toast.success(t('messages.deleted'));
      }
    );
  };
  const confirmBatchDelete = () =>
    mutation.run(
      'delete:batch',
      () => deleteNotices(state.table.selected),
      () => {
        updatePageAfterNoticeBatchDelete({
          table: state.table,
          totalRowsInPage: resource.items.length,
          totalRowsFiltered: resource.total,
        });
        state.setBatchDeleteOpen(false);
        state.table.setSelected([]);
        toast.success(t('messages.deleted'));
      }
    );
  return { confirmDelete, confirmBatchDelete };
}
