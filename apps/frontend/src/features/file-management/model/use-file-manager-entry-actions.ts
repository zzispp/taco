'use client';

import type { FileEntry } from 'src/entities/file';
import type { FileManagerActionOptions } from './file-manager-action-options';

import { useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';

import { ROOT_DIRECTORY_ID } from './constants';
import { canMoveToDestination } from './move-destinations';
import {
  canEditFileEntry,
  canDeleteFileEntry,
  canPreviewFileEntry,
  canRestoreFileEntry,
  canDownloadFileEntry,
  canUseFileBatchAction,
} from './file-action-policy';
import {
  moveFile,
  trashFile,
  updateFile,
  trashFiles,
  restoreFile,
  downloadFile,
  restoreFiles,
  openFilePreview,
  permanentlyDeleteFile,
  permanentlyDeleteFiles,
} from '../api';

type EntryActionNavigation = Readonly<{
  closeDelete: () => void;
  closeMove: () => void;
  closeBatchAction: () => void;
}>;

export function useFileManagerEntryActions(
  options: FileManagerActionOptions,
  navigation: EntryActionNavigation
) {
  return {
    previewEntry: usePreviewAction(options),
    downloadEntry: useDownloadAction(options),
    deleteEntry: useDeleteAction(options, navigation.closeDelete),
    restoreEntry: useRestoreAction(options),
    renameEntry: useRenameAction(options),
    updateTags: useTagAction(options),
    moveEntry: useMoveAction(options, navigation.closeMove),
    executeBatchAction: useBatchAction(options, navigation.closeBatchAction),
  };
}

function usePreviewAction(options: FileManagerActionOptions) {
  return useCallback(
    (entry: FileEntry) => {
      if (!canPreviewFileEntry(options.state.mode, options.permissions, entry)) return;
      void options.mutation.run({
        key: `preview:${entry.id}`,
        failureMessage: options.t('file.messages.previewFailed'),
        action: () => openFilePreview(entry.id),
      });
    },
    [options]
  );
}

function useDownloadAction(options: FileManagerActionOptions) {
  return useCallback(
    (entry: FileEntry) => {
      if (!canDownloadFileEntry(options.state.mode, options.permissions)) return;
      void options.mutation.run({
        key: `download:${entry.id}`,
        failureMessage: options.t('file.messages.downloadFailed'),
        action: () => downloadFile(entry),
      });
    },
    [options]
  );
}

function useDeleteAction(options: FileManagerActionOptions, closeDelete: () => void) {
  return useCallback(() => {
    const entry = options.state.deleteTarget;
    if (!entry || !canDeleteFileEntry(options.state.mode, options.permissions)) return;
    const action =
      options.state.mode === 'trash'
        ? () => permanentlyDeleteFile(entry.id)
        : () => trashFile(entry.id);
    void options.mutation.run({
      key: `delete:${entry.id}`,
      failureMessage: options.t('file.messages.deleteFailed'),
      action,
      onSuccess: () => {
        closeDelete();
        options.state.setDetailId(null);
        const key = options.state.mode === 'trash' ? 'permanentlyDeleted' : 'trashed';
        toast.success(options.t(`file.messages.${key}`));
      },
    });
  }, [closeDelete, options]);
}

function useRestoreAction(options: FileManagerActionOptions) {
  return useCallback(
    (entry: FileEntry) => {
      if (!canRestoreFileEntry(options.state.mode, options.permissions)) return;
      void options.mutation.run({
        key: `restore:${entry.id}`,
        failureMessage: options.t('file.messages.restoreFailed'),
        action: () => restoreFile(entry.id),
        onSuccess: () => {
          toast.success(options.t('file.messages.restored'));
        },
      });
    },
    [options]
  );
}

function useRenameAction(options: FileManagerActionOptions) {
  return useCallback(
    (entry: FileEntry, name: string) => {
      if (!name.trim() || !canEditFileEntry(options.state.mode, options.permissions)) return;
      void options.mutation.run({
        key: `rename:${entry.id}`,
        failureMessage: options.t('file.messages.renameFailed'),
        action: () => updateFile(entry.id, { name: name.trim() }),
        onSuccess: () => {
          toast.success(options.t('file.messages.renamed'));
        },
      });
    },
    [options]
  );
}

function useTagAction(options: FileManagerActionOptions) {
  return useCallback(
    (entry: FileEntry, tags: string[]) => {
      if (!canEditFileEntry(options.state.mode, options.permissions)) return;
      void options.mutation.run({
        key: `tags:${entry.id}`,
        failureMessage: options.t('file.messages.tagsFailed'),
        action: () => updateFile(entry.id, { tags }),
        onSuccess: () => {
          toast.success(options.t('file.messages.tagsUpdated'));
        },
      });
    },
    [options]
  );
}

function useMoveAction(options: FileManagerActionOptions, closeMove: () => void) {
  return useCallback(() => {
    const entry = options.state.moveTarget;
    const destination = options.state.moveDestinationId;
    if (
      !entry ||
      !canEditFileEntry(options.state.mode, options.permissions) ||
      !canMoveToDestination(destination, entry)
    ) {
      return;
    }
    const parentId = destination === ROOT_DIRECTORY_ID ? null : destination;
    void options.mutation.run({
      key: `move:${entry.id}`,
      failureMessage: options.t('file.messages.moveFailed'),
      action: () => moveFile(entry.id, parentId),
      onSuccess: () => {
        closeMove();
        options.state.setDetailId(null);
        options.state.table.setSelected([]);
        toast.success(options.t('file.messages.moved'));
      },
    });
  }, [closeMove, options]);
}

function useBatchAction(options: FileManagerActionOptions, closeBatchAction: () => void) {
  return useCallback(() => {
    const action = options.state.batchAction;
    const ids = options.state.table.selected;
    if (
      !action ||
      !ids.length ||
      !canUseFileBatchAction(options.state.mode, options.permissions, action)
    ) {
      return;
    }
    const run = batchMutation(action, ids);
    void options.mutation.run({
      key: `batch:${action}`,
      failureMessage: options.t(`file.messages.${action}BatchFailed`),
      action: run,
      onSuccess: () => {
        closeBatchAction();
        options.state.table.setSelected([]);
        toast.success(options.t(`file.messages.${action}BatchCompleted`));
      },
    });
  }, [closeBatchAction, options]);
}

function batchMutation(
  action: NonNullable<FileManagerActionOptions['state']['batchAction']>,
  ids: string[]
) {
  if (action === 'trash') return () => trashFiles(ids);
  if (action === 'restore') return () => restoreFiles(ids);
  return () => permanentlyDeleteFiles(ids);
}
