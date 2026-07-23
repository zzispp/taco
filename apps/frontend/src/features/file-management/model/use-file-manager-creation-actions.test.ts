import type { UploadQueueItem } from './upload-queue';
import type { FileManagerActionOptions } from './file-manager-action-options';

import { vi, it, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({
  createFolder: vi.fn(),
  sha256File: vi.fn(),
  uploadFile: vi.fn(),
  closeUpload: vi.fn(),
  closeMoveFolderDialog: vi.fn(),
  refreshMoveFolders: vi.fn(),
}));

vi.mock('react', () => ({
  useCallback: <T>(callback: T) => callback,
  useRef: <T>(current: T) => ({ current }),
}));

vi.mock('src/shared/ui/snackbar', () => ({
  toast: { error: vi.fn(), info: vi.fn(), success: vi.fn() },
}));

vi.mock('../lib/sha256', () => ({ sha256File: state.sha256File }));

vi.mock('../api', () => ({
  createFolder: state.createFolder,
  uploadFile: state.uploadFile,
  UploadCancellationError: class UploadCancellationError extends Error {},
}));

import { useFileManagerCreationActions } from './use-file-manager-creation-actions';

describe('file manager creation actions', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('creates the dropped folder hierarchy once and uploads each file below its resolved parent', async () => {
    const architecture = { id: 'folder-architecture' };
    const api = { id: 'folder-api' };
    state.createFolder.mockResolvedValueOnce(architecture).mockResolvedValueOnce(api);
    state.sha256File.mockResolvedValueOnce('digest-service').mockResolvedValueOnce('digest-notes');
    state.uploadFile.mockResolvedValue({ id: 'uploaded' });
    const actions = useFileManagerCreationActions(
      actionOptions([item('Architecture/api/service.md'), item('Architecture/notes.md')]),
      {
        closeUpload: state.closeUpload,
        closeFolderDialog: vi.fn(),
        closeMoveFolderDialog: vi.fn(),
      }
    );

    actions.submitUpload();

    await vi.waitFor(() => expect(state.closeUpload).toHaveBeenCalledOnce());

    expect(state.createFolder).toHaveBeenNthCalledWith(
      1,
      { space_id: 'space-1', parent_id: 'current-folder', name: 'Architecture' },
      { refresh: false }
    );
    expect(state.createFolder).toHaveBeenNthCalledWith(
      2,
      { space_id: 'space-1', parent_id: 'folder-architecture', name: 'api' },
      { refresh: false }
    );
    expect(state.uploadFile).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({ name: 'service.md' }),
      { space_id: 'space-1', parent_id: 'folder-api' },
      expect.objectContaining({ digest: 'digest-service' })
    );
    expect(state.uploadFile).toHaveBeenNthCalledWith(
      2,
      expect.objectContaining({ name: 'notes.md' }),
      { space_id: 'space-1', parent_id: 'folder-architecture' },
      expect.objectContaining({ digest: 'digest-notes' })
    );
  });

  it('creates a new folder inside the selected move destination', async () => {
    const run = vi.fn(
      async ({
        action,
        onSuccess,
      }: {
        action: () => Promise<unknown>;
        onSuccess: () => void | Promise<void>;
      }) => {
        await action();
        await onSuccess();
      }
    );
    state.createFolder.mockResolvedValue({ id: 'new-folder' });
    state.refreshMoveFolders.mockResolvedValue(undefined);
    const actions = useFileManagerCreationActions(moveFolderActionOptions(run), {
      closeUpload: state.closeUpload,
      closeFolderDialog: vi.fn(),
      closeMoveFolderDialog: state.closeMoveFolderDialog,
    });

    actions.submitMoveFolder();

    await vi.waitFor(() =>
      expect(state.createFolder).toHaveBeenCalledWith({
        space_id: 'target-space',
        parent_id: 'destination-folder',
        name: '归档',
      })
    );
    await vi.waitFor(() => {
      expect(state.refreshMoveFolders).toHaveBeenCalledOnce();
      expect(state.closeMoveFolderDialog).toHaveBeenCalledOnce();
    });
  });
});

function actionOptions(uploadItems: readonly UploadQueueItem[]): FileManagerActionOptions {
  return {
    state: {
      uploadItems,
      parentId: 'current-folder',
      folderName: '',
      updateUploadItem: vi.fn(),
    },
    mutation: { run: vi.fn() },
    permissions: { canUpload: true, canAddFolder: true },
    spaceId: 'space-1',
    refreshMoveFolders: state.refreshMoveFolders,
    t: (key: string) => key,
  } as unknown as FileManagerActionOptions;
}

function moveFolderActionOptions(run: ReturnType<typeof vi.fn>): FileManagerActionOptions {
  return {
    state: {
      uploadItems: [],
      parentId: 'current-folder',
      folderName: '',
      moveFolderName: '  归档  ',
      moveDestinationId: 'destination-folder',
      moveTarget: { space_id: 'target-space' },
      updateUploadItem: vi.fn(),
    },
    mutation: { run },
    permissions: { canAddFolder: true, canUpload: false },
    spaceId: 'space-1',
    refreshMoveFolders: state.refreshMoveFolders,
    t: (key: string) => key,
  } as unknown as FileManagerActionOptions;
}

function item(relativePath: string): UploadQueueItem {
  const name = relativePath.split('/').at(-1) ?? 'unnamed';
  return {
    id: relativePath,
    file: new File(['content'], name, { type: 'text/plain' }),
    relativePath,
    digest: null,
    progress: null,
    status: 'queued',
  };
}
