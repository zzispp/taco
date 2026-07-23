'use client';

import type { FileEntry } from 'src/entities/file';
import type { FileBatchAction } from './constants';
import type { FileManagerActionOptions } from './file-manager-action-options';

import { useCallback } from 'react';

import { ROOT_DIRECTORY_ID } from './constants';
import { resetFileFilters } from './use-file-manager-state';
import { fileDirectoryParentTrail } from './directory-navigation';
import { useFileManagerEntryActions } from './use-file-manager-entry-actions';
import { useFileManagerCreationActions } from './use-file-manager-creation-actions';

type ActionOptions = FileManagerActionOptions;

export function useFileManagerActions(options: ActionOptions) {
  const navigation = useNavigationActions(options);
  const creation = useFileManagerCreationActions(options, navigation);
  const mutation = useFileManagerEntryActions(options, navigation);
  return { ...navigation, ...creation, ...mutation };
}

function useNavigationActions(options: ActionOptions) {
  const filters = useFilterNavigation(options.state);
  const directories = useDirectoryNavigation(options);
  const dialogs = useEntryDialogNavigation(options.state);
  const moves = useMoveDialogNavigation(options.state);
  const batch = useBatchDialogNavigation(options.state);
  return { ...filters, ...directories, ...dialogs, ...moves, ...batch };
}

function useFilterNavigation(state: ActionOptions['state']) {
  return {
    applyFilters: useCallback(() => {
      state.setFilters(state.filterDraft);
      state.table.onResetCursor();
    }, [state]),
    resetFilters: useCallback(() => resetFileFilters(state), [state]),
  };
}

function useDirectoryNavigation(options: ActionOptions) {
  const { state } = options;
  return {
    openFolder: useCallback(
      (id: string) => {
        state.enterDirectory(id);
        state.table.onResetCursor();
      },
      [state]
    ),
    goToParentFolder: useCallback(() => {
      if (!state.parentId || !options.directoryTrail.length) return;
      state.replaceDirectoryTrail(fileDirectoryParentTrail(options.directoryTrail));
      state.table.onResetCursor();
    }, [options.directoryTrail, state]),
    goToDirectory: useCallback(
      (directoryTrail: readonly string[]) => {
        state.replaceDirectoryTrail(directoryTrail);
        state.table.onResetCursor();
      },
      [state]
    ),
  };
}

function useEntryDialogNavigation(state: ActionOptions['state']) {
  return {
    openUpload: useCallback(() => state.openUploadDialog(), [state]),
    closeUpload: useCallback(() => state.closeUploadDialog(), [state]),
    openFolderDialog: useCallback(() => state.setFolderOpen(true), [state]),
    closeFolderDialog: useCallback(() => {
      state.setFolderOpen(false);
      state.setFolderName('');
    }, [state]),
    openDetail: useCallback((entry: FileEntry) => state.setDetailId(entry.id), [state]),
    closeDetail: useCallback(() => state.setDetailId(null), [state]),
    requestDelete: useCallback((entry: FileEntry) => state.setDeleteTarget(entry), [state]),
    closeDelete: useCallback(() => state.setDeleteTarget(null), [state]),
  };
}

function useMoveDialogNavigation(state: ActionOptions['state']) {
  return {
    requestMove: useCallback(
      (entry: FileEntry) => {
        state.setMoveTarget(entry);
        state.setMoveDestinationId(entry.parent_id ?? ROOT_DIRECTORY_ID);
      },
      [state]
    ),
    openMoveFolderDialog: useCallback(() => state.openMoveFolderDialog(), [state]),
    closeMoveFolderDialog: useCallback(() => state.closeMoveFolderDialog(), [state]),
    closeMove: useCallback(() => {
      state.closeMoveFolderDialog();
      state.setMoveTarget(null);
      state.setMoveDestinationId(ROOT_DIRECTORY_ID);
    }, [state]),
  };
}

function useBatchDialogNavigation(state: ActionOptions['state']) {
  return {
    requestBatchAction: useCallback(
      (action: FileBatchAction) => state.setBatchAction(action),
      [state]
    ),
    closeBatchAction: useCallback(() => state.setBatchAction(null), [state]),
  };
}
