import type { FileEntry } from 'src/entities/file';
import type { UploadQueueItem } from './upload-queue';
import type { FileManagerRoute } from './file-manager-route';

import { useState, useReducer } from 'react';

import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { createUploadQueue } from './upload-queue';
import { EMPTY_FILE_MANAGER_ROUTE } from './file-manager-route';
import {
  type FileViewMode,
  ROOT_DIRECTORY_ID,
  DEFAULT_FILE_FILTERS,
  type FileFilterDraft,
  type FileManagerMode,
  type FileBatchAction,
} from './constants';

export function useFileManagerState(route: FileManagerRoute = EMPTY_FILE_MANAGER_ROUTE) {
  const [viewMode, setViewMode] = useState<FileViewMode>('list');
  const [mode, setMode] = useState<FileManagerMode>('active');
  const [filters, setFilters] = useState(DEFAULT_FILE_FILTERS);
  const [filterDraft, setFilterDraft] = useState(DEFAULT_FILE_FILTERS);
  const [space, dispatchSpace] = useReducer(
    fileManagerSpaceReducer,
    route,
    createFileManagerSpaceStateFromRoute
  );
  const parentId = fileManagerParentId(space);
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'updated_at',
    defaultOrder: 'desc',
    scopeKey: fileManagerScopeKey({
      spaceId: space.spaceId,
      mode,
      parentId,
      filters,
    }),
  });
  const spaceSetters = createFileManagerSpaceSetters({
    space,
    dispatch: dispatchSpace,
    resetTable: table.onResetCursor,
  });

  return {
    table,
    ...space,
    parentId,
    viewMode,
    mode,
    filters,
    filterDraft,
    setViewMode,
    setMode: (next: FileManagerMode) => {
      setMode(next);
      spaceSetters.resetDirectory();
      table.onResetCursor();
    },
    setFilters,
    setFilterDraft,
    ...spaceSetters,
  };
}

function createFileManagerSpaceSetters(
  options: Readonly<{
    space: FileManagerSpaceState;
    dispatch: (action: FileManagerSpaceAction) => void;
    resetTable: () => void;
  }>
) {
  const update = (patch: Partial<FileManagerSpaceState>) =>
    options.dispatch({ type: 'update', patch });
  return {
    enterDirectory: (directoryId: string) =>
      options.dispatch({ type: 'enter-directory', directoryId }),
    leaveDirectory: () => options.dispatch({ type: 'leave-directory' }),
    replaceDirectoryTrail: (directoryTrail: readonly string[]) =>
      options.dispatch({ type: 'replace-directory-trail', directoryTrail }),
    resetDirectory: () => options.dispatch({ type: 'reset-directory' }),
    openUploadDialog: () => options.dispatch({ type: 'open-upload' }),
    closeUploadDialog: () => options.dispatch({ type: 'close-upload' }),
    appendUploadFiles: (files: readonly File[]) =>
      options.dispatch({ type: 'append-upload-files', files }),
    removeUploadItem: (id: string) => options.dispatch({ type: 'remove-upload-item', id }),
    updateUploadItem: (id: string, patch: UploadQueueItemPatch) =>
      options.dispatch({ type: 'update-upload-item', id, patch }),
    setDetailId: (detailId: string | null) => update({ detailId }),
    setFolderOpen: (folderOpen: boolean) => update({ folderOpen }),
    setFolderName: (folderName: string) => update({ folderName }),
    setDeleteTarget: (deleteTarget: FileEntry | null) => update({ deleteTarget }),
    setMoveTarget: (moveTarget: FileEntry | null) => update({ moveTarget }),
    setMoveDestinationId: (moveDestinationId: string) => update({ moveDestinationId }),
    openMoveFolderDialog: () => update({ moveFolderOpen: true, moveFolderName: '' }),
    closeMoveFolderDialog: () => update({ moveFolderOpen: false, moveFolderName: '' }),
    setMoveFolderName: (moveFolderName: string) => update({ moveFolderName }),
    setBatchAction: (batchAction: FileBatchAction | null) => update({ batchAction }),
    setSpaceId: (nextSpaceId: string | undefined) =>
      changeFileManagerSpace({
        currentSpaceId: options.space.spaceId,
        nextSpaceId,
        dispatch: options.dispatch,
        resetTable: options.resetTable,
      }),
  };
}

export type FileManagerSpaceState = Readonly<{
  spaceId: string | undefined;
  directoryTrail: readonly string[];
  uploadOpen: boolean;
  uploadItems: readonly UploadQueueItem[];
  detailId: string | null;
  folderOpen: boolean;
  folderName: string;
  deleteTarget: FileEntry | null;
  moveTarget: FileEntry | null;
  moveDestinationId: string;
  moveFolderOpen: boolean;
  moveFolderName: string;
  batchAction: FileBatchAction | null;
}>;

export type FileManagerSpaceAction =
  | Readonly<{ type: 'select-space'; spaceId: string | undefined }>
  | Readonly<{ type: 'enter-directory'; directoryId: string }>
  | Readonly<{ type: 'leave-directory' }>
  | Readonly<{ type: 'replace-directory-trail'; directoryTrail: readonly string[] }>
  | Readonly<{ type: 'reset-directory' }>
  | Readonly<{ type: 'open-upload' }>
  | Readonly<{ type: 'close-upload' }>
  | Readonly<{ type: 'append-upload-files'; files: readonly File[] }>
  | Readonly<{ type: 'remove-upload-item'; id: string }>
  | Readonly<{ type: 'update-upload-item'; id: string; patch: UploadQueueItemPatch }>
  | Readonly<{ type: 'update'; patch: Partial<FileManagerSpaceState> }>;

export type UploadQueueItemPatch = Readonly<
  Partial<Pick<UploadQueueItem, 'digest' | 'progress' | 'status'>>
>;

export function createFileManagerSpaceState(spaceId?: string): FileManagerSpaceState {
  return {
    spaceId,
    directoryTrail: [],
    uploadOpen: false,
    uploadItems: [],
    detailId: null,
    folderOpen: false,
    folderName: '',
    deleteTarget: null,
    moveTarget: null,
    moveDestinationId: ROOT_DIRECTORY_ID,
    moveFolderOpen: false,
    moveFolderName: '',
    batchAction: null,
  };
}

export function createFileManagerSpaceStateFromRoute(
  route: FileManagerRoute
): FileManagerSpaceState {
  return {
    ...createFileManagerSpaceState(route.spaceId ?? undefined),
    directoryTrail: route.parentId ? [route.parentId] : [],
    detailId: route.detailId,
  };
}

export function fileManagerSpaceReducer(
  state: FileManagerSpaceState,
  action: FileManagerSpaceAction
): FileManagerSpaceState {
  if (action.type === 'select-space') return createFileManagerSpaceState(action.spaceId);
  if (action.type === 'enter-directory') {
    return { ...state, directoryTrail: [...state.directoryTrail, action.directoryId] };
  }
  if (action.type === 'leave-directory') {
    return { ...state, directoryTrail: state.directoryTrail.slice(0, -1) };
  }
  if (action.type === 'replace-directory-trail') {
    if (sameDirectoryTrail(state.directoryTrail, action.directoryTrail)) return state;
    return { ...state, directoryTrail: [...action.directoryTrail] };
  }
  if (action.type === 'reset-directory') return { ...state, directoryTrail: [] };
  if (action.type === 'open-upload') return { ...state, uploadOpen: true, uploadItems: [] };
  if (action.type === 'close-upload') return { ...state, uploadOpen: false, uploadItems: [] };
  if (action.type === 'append-upload-files') {
    return { ...state, uploadItems: [...state.uploadItems, ...createUploadQueue(action.files)] };
  }
  if (action.type === 'remove-upload-item') {
    return { ...state, uploadItems: state.uploadItems.filter((item) => item.id !== action.id) };
  }
  if (action.type === 'update-upload-item') {
    return {
      ...state,
      uploadItems: state.uploadItems.map((item) =>
        item.id === action.id ? { ...item, ...action.patch } : item
      ),
    };
  }
  return { ...state, ...action.patch };
}

function sameDirectoryTrail(left: readonly string[], right: readonly string[]) {
  return left.length === right.length && left.every((id, index) => id === right[index]);
}

export function fileManagerParentId(state: Pick<FileManagerSpaceState, 'directoryTrail'>) {
  return state.directoryTrail.at(-1) ?? null;
}

export function changeFileManagerSpace(
  options: Readonly<{
    currentSpaceId: string | undefined;
    nextSpaceId: string | undefined;
    dispatch: (action: FileManagerSpaceAction) => void;
    resetTable: () => void;
  }>
) {
  if (options.currentSpaceId === options.nextSpaceId) return;
  options.dispatch({ type: 'select-space', spaceId: options.nextSpaceId });
  options.resetTable();
}

export function fileManagerScopeKey(
  options: Readonly<{
    spaceId?: string;
    mode: FileManagerMode;
    parentId: string | null;
    filters: FileFilterDraft;
  }>
) {
  return JSON.stringify([options.spaceId ?? null, options.mode, options.parentId, options.filters]);
}

export type FileManagerState = ReturnType<typeof useFileManagerState>;

export function hasActiveFileFilters(filters: FileFilterDraft) {
  return Object.values(filters).some(Boolean);
}

export function resetFileFilters(state: FileManagerState) {
  state.setFilterDraft(DEFAULT_FILE_FILTERS);
  state.setFilters(DEFAULT_FILE_FILTERS);
  state.table.onResetCursor();
}
