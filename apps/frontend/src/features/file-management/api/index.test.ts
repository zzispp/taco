import { mutate } from 'swr';
import { it, vi, expect, describe, beforeEach } from 'vitest';

const { invalidateFilePreviewsMock } = vi.hoisted(() => ({
  invalidateFilePreviewsMock: vi.fn(),
}));

import axios from 'src/shared/api/http-client';

import { fileEndpoints } from 'src/entities/file';

import {
  moveFile,
  trashFile,
  trashFiles,
  updateFileSpace,
  permanentlyDeleteFile,
  permanentlyDeleteFiles,
} from './index';

vi.mock('swr', () => ({ mutate: vi.fn() }));
vi.mock('src/shared/api/http-client', () => ({
  default: { delete: vi.fn(), post: vi.fn(), put: vi.fn() },
}));
vi.mock('./preview', () => ({
  invalidateFilePreviews: invalidateFilePreviewsMock,
  openFilePreview: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(mutate).mockResolvedValue(undefined);
});

describe('file space mutations', () => {
  it('updates a concrete space quota and refreshes file resources', async () => {
    const payload = { quota_bytes: 32 * 1024 ** 3 };
    vi.mocked(axios.put).mockResolvedValueOnce({ data: { id: 'space/1' } } as never);

    await updateFileSpace('space/1', payload);

    expect(axios.put).toHaveBeenCalledExactlyOnceWith(fileEndpoints.space('space/1'), payload);
    expect(mutate).toHaveBeenCalledTimes(5);
  });

  it('sends null to restore the provider-independent default quota', async () => {
    vi.mocked(axios.put).mockResolvedValueOnce({ data: { id: 'space-1' } } as never);

    await updateFileSpace('space-1', { quota_bytes: null });

    expect(axios.put).toHaveBeenCalledExactlyOnceWith(fileEndpoints.space('space-1'), {
      quota_bytes: null,
    });
  });
});

describe('file move mutation', () => {
  it('uses null as the explicit root destination', async () => {
    vi.mocked(axios.put).mockResolvedValueOnce({ data: { id: 'file-1' } } as never);

    await moveFile('file-1', null);

    expect(axios.put).toHaveBeenCalledExactlyOnceWith(fileEndpoints.asset('file-1'), {
      parent_id: null,
    });
  });
});

describe('file lifecycle mutations', () => {
  it('closes matching previews only after moving one asset into the recycle bin', async () => {
    vi.mocked(axios.post).mockResolvedValueOnce({} as never);

    await trashFile('file-1');

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(fileEndpoints.trash('file-1'));
    expect(invalidateFilePreviewsMock).toHaveBeenCalledExactlyOnceWith(['file-1']);
    expect(mutate).toHaveBeenCalledTimes(5);
  });

  it('closes matching previews after a batch move into the recycle bin', async () => {
    vi.mocked(axios.post).mockResolvedValueOnce({} as never);

    await trashFiles(['file-1', 'file-2']);

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(fileEndpoints.batchTrash, {
      ids: ['file-1', 'file-2'],
    });
    expect(invalidateFilePreviewsMock).toHaveBeenCalledExactlyOnceWith(['file-1', 'file-2']);
  });

  it('keeps previews open when moving an asset into the recycle bin fails', async () => {
    vi.mocked(axios.post).mockRejectedValueOnce(new Error('request failed'));

    await expect(trashFile('file-1')).rejects.toThrow('request failed');

    expect(invalidateFilePreviewsMock).not.toHaveBeenCalled();
  });

  it('closes matching previews after permanent deletion', async () => {
    vi.mocked(axios.delete).mockResolvedValueOnce({} as never);

    await permanentlyDeleteFile('file-1');

    expect(axios.delete).toHaveBeenCalledExactlyOnceWith(fileEndpoints.purge('file-1'));
    expect(invalidateFilePreviewsMock).toHaveBeenCalledExactlyOnceWith(['file-1']);
  });

  it('closes matching previews after batch permanent deletion', async () => {
    vi.mocked(axios.post).mockResolvedValueOnce({} as never);

    await permanentlyDeleteFiles(['file-1', 'file-2']);

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(fileEndpoints.batchPurge, {
      ids: ['file-1', 'file-2'],
    });
    expect(invalidateFilePreviewsMock).toHaveBeenCalledExactlyOnceWith(['file-1', 'file-2']);
  });
});
