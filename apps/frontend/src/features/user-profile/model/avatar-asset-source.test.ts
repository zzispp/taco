import { it, vi, expect, describe } from 'vitest';

const { getFilePreviewBlobMock } = vi.hoisted(() => ({ getFilePreviewBlobMock: vi.fn() }));

vi.mock('src/entities/file', () => ({ getFilePreviewBlob: getFilePreviewBlobMock }));

import { loadAvatarAssetPreview } from './avatar-asset-source';

describe('avatar asset source', () => {
  it('loads a crop source through the query-authorized preview endpoint', async () => {
    const signal = new AbortController().signal;
    const preview = new Blob(['avatar'], { type: 'image/png' });
    getFilePreviewBlobMock.mockResolvedValueOnce(preview);

    await expect(loadAvatarAssetPreview('asset-1', signal)).resolves.toBe(preview);
    expect(getFilePreviewBlobMock).toHaveBeenCalledExactlyOnceWith('asset-1', signal);
  });
});
