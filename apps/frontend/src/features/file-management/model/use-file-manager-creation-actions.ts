'use client';

import type { UploadQueueItem } from './upload-queue';
import type { FileManagerActionOptions } from './file-manager-action-options';

import { useRef, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { apiMutationErrorMessage } from 'src/shared/api/mutation-error';

import { sha256File } from '../lib/sha256';
import { ROOT_DIRECTORY_ID, FILE_HASH_CHUNK_BYTES } from './constants';
import { uploadFile, createFolder, UploadCancellationError } from '../api';
import { canSubmitUploadQueue, uploadParentSegments } from './upload-queue';

type CreationNavigation = Readonly<{
  closeUpload: () => void;
  closeFolderDialog: () => void;
  closeMoveFolderDialog: () => void;
}>;

type FolderCreationRequest = Readonly<{
  spaceId?: string;
  parentId: string | null;
  name: string;
  mutationKey: string;
  close: () => void;
  refresh?: () => Promise<void>;
}>;

type UploadExecution = Readonly<{
  options: FileManagerActionOptions;
  controller: AbortController;
  spaceId: string;
  parentId: string | null;
  items: readonly UploadQueueItem[];
  directories: Map<string, string>;
}>;

export function useFileManagerCreationActions(
  options: FileManagerActionOptions,
  navigation: CreationNavigation
) {
  const folder = useFolderCreationAction(options, navigation);
  const upload = useUploadActions(options, navigation);
  return { ...folder, ...upload };
}

function useFolderCreationAction(
  options: FileManagerActionOptions,
  navigation: CreationNavigation
) {
  const submitFolder = useFolderSubmit(options, {
    spaceId: options.spaceId,
    parentId: options.state.parentId,
    name: options.state.folderName,
    mutationKey: 'folder:create',
    close: navigation.closeFolderDialog,
  });
  const submitMoveFolder = useFolderSubmit(options, {
    spaceId: options.state.moveTarget?.space_id ?? options.spaceId,
    parentId:
      options.state.moveDestinationId === ROOT_DIRECTORY_ID
        ? null
        : options.state.moveDestinationId,
    name: options.state.moveFolderName,
    mutationKey: 'move-folder:create',
    close: navigation.closeMoveFolderDialog,
    refresh: options.refreshMoveFolders,
  });
  return { submitFolder, submitMoveFolder };
}

function useFolderSubmit(options: FileManagerActionOptions, request: FolderCreationRequest) {
  return useCallback(() => {
    const spaceId = request.spaceId;
    const name = request.name.trim();
    if (!spaceId || !name || !options.permissions.canAddFolder) return;
    void options.mutation.run({
      key: request.mutationKey,
      failureMessage: options.t('file.messages.createFolderFailed'),
      action: () => createFolder({ space_id: spaceId, parent_id: request.parentId, name }),
      onSuccess: async () => {
        await request.refresh?.();
        request.close();
        toast.success(options.t('file.messages.folderCreated'));
      },
    });
  }, [options, request]);
}

function useUploadActions(options: FileManagerActionOptions, navigation: CreationNavigation) {
  const uploadController = useRef<AbortController | null>(null);
  const submitUpload = useCallback(() => {
    const execution = createUploadExecution(options);
    if (!execution || uploadController.current) return;
    const controller = new AbortController();
    uploadController.current = controller;
    const activeExecution = { ...execution, controller };
    void uploadQueue(activeExecution)
      .then(() => {
        navigation.closeUpload();
        toast.success(options.t('file.messages.uploaded'));
      })
      .catch((error) => notifyUploadFailure(error, controller, options.t))
      .finally(() => {
        if (uploadController.current === controller) uploadController.current = null;
      });
  }, [navigation, options]);
  const cancelUpload = useCallback(() => uploadController.current?.abort(), []);
  return { submitUpload, cancelUpload };
}

function createUploadExecution(
  options: FileManagerActionOptions
): Omit<UploadExecution, 'controller'> | null {
  const items = options.state.uploadItems.filter(canUploadQueueItem);
  const spaceId = options.spaceId;
  if (!spaceId || !options.permissions.canUpload || !canSubmitUploadQueue(items)) return null;
  return { options, spaceId, parentId: options.state.parentId, items, directories: new Map() };
}

function canUploadQueueItem(item: UploadQueueItem) {
  return item.status === 'queued' || item.status === 'failed';
}

async function uploadQueue(execution: UploadExecution) {
  for (const item of execution.items) {
    await uploadQueueItem({ execution, item });
  }
}

async function uploadQueueItem({
  execution,
  item,
}: Readonly<{
  execution: UploadExecution;
  item: UploadQueueItem;
}>) {
  try {
    const digest = await calculateUploadItemDigest({ execution, item });
    const parentId = await resolveUploadParent({ execution, item });
    await uploadFile(
      item.file,
      { space_id: execution.spaceId, parent_id: parentId },
      {
        digest,
        signal: execution.controller.signal,
        onProgress: (progress) =>
          execution.options.state.updateUploadItem(item.id, {
            progress,
            status: progress.phase,
          }),
      }
    );
    execution.options.state.updateUploadItem(item.id, {
      progress: { phase: 'completing', completedBytes: item.file.size, totalBytes: item.file.size },
      status: 'completed',
    });
  } catch (error) {
    execution.options.state.updateUploadItem(item.id, {
      progress: null,
      status: uploadItemFailureStatus(error, execution.controller),
    });
    throw error;
  }
}

async function calculateUploadItemDigest({
  execution,
  item,
}: Readonly<{
  execution: UploadExecution;
  item: UploadQueueItem;
}>): Promise<string> {
  execution.options.state.updateUploadItem(item.id, {
    progress: { phase: 'hashing', completedBytes: 0, totalBytes: item.file.size },
    status: 'hashing',
  });
  const digest = await sha256File(item.file, {
    chunkSize: FILE_HASH_CHUNK_BYTES,
    signal: execution.controller.signal,
    onProgress: (completedBytes) =>
      execution.options.state.updateUploadItem(item.id, {
        progress: { phase: 'hashing', completedBytes, totalBytes: item.file.size },
        status: 'hashing',
      }),
  });
  execution.options.state.updateUploadItem(item.id, { digest, progress: null, status: 'queued' });
  return digest;
}

async function resolveUploadParent({
  execution,
  item,
}: Readonly<{
  execution: UploadExecution;
  item: UploadQueueItem;
}>): Promise<string | null> {
  let parentId = execution.parentId;
  let path = '';
  for (const segment of uploadParentSegments(item)) {
    path = path ? `${path}/${segment}` : segment;
    const knownDirectory = execution.directories.get(path);
    if (knownDirectory) {
      parentId = knownDirectory;
      continue;
    }
    const directory = await createFolder(
      { space_id: execution.spaceId, parent_id: parentId, name: segment },
      { refresh: false }
    );
    parentId = directory.id;
    execution.directories.set(path, directory.id);
  }
  return parentId;
}

function uploadItemFailureStatus(error: unknown, controller: AbortController) {
  if (controller.signal.aborted && !(error instanceof UploadCancellationError))
    return 'queued' as const;
  return 'failed' as const;
}

function notifyUploadFailure(
  error: unknown,
  controller: AbortController,
  t: FileManagerActionOptions['t']
) {
  if (error instanceof UploadCancellationError) {
    toast.error(t('file.messages.uploadCancelFailed'));
    return;
  }
  if (controller.signal.aborted) {
    toast.info(t('file.messages.uploadCanceled'));
    return;
  }
  toast.error(apiMutationErrorMessage(error, t('file.messages.uploadFailed')));
}
