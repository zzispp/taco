import { it, vi, expect, describe, beforeEach } from 'vitest';

import axios from 'src/shared/api/http-client';

import { fileEndpoints } from './endpoints';
import { getFileContentBlob, getFilePreviewBlob, getFileThumbnailBlob } from './content';

vi.mock('src/shared/api/http-client', () => ({
  default: { get: vi.fn() },
}));

describe('file content API', () => {
  beforeEach(() => vi.clearAllMocks());

  it('reads content through the authenticated file endpoint as a Blob', async () => {
    const signal = new AbortController().signal;
    const blob = new Blob(['image'], { type: 'image/png' });
    vi.mocked(axios.get).mockResolvedValueOnce({ data: blob } as never);

    await expect(getFileContentBlob('file/1', signal)).resolves.toBe(blob);
    expect(axios.get).toHaveBeenCalledExactlyOnceWith(fileEndpoints.content('file/1'), {
      responseType: 'blob',
      signal,
    });
  });

  it('does not add an undefined abort signal', async () => {
    const blob = new Blob(['image'], { type: 'image/png' });
    vi.mocked(axios.get).mockResolvedValueOnce({ data: blob } as never);

    await getFileContentBlob('file-1');

    expect(axios.get).toHaveBeenCalledExactlyOnceWith(fileEndpoints.content('file-1'), {
      responseType: 'blob',
    });
  });

  it('reads inline previews through the authenticated preview endpoint', async () => {
    const blob = new Blob(['preview'], { type: 'text/plain' });
    vi.mocked(axios.get).mockResolvedValueOnce({ data: blob } as never);

    await expect(getFilePreviewBlob('file/preview')).resolves.toBe(blob);
    expect(axios.get).toHaveBeenCalledExactlyOnceWith(fileEndpoints.preview('file/preview'), {
      responseType: 'blob',
    });
  });

  it('reads thumbnails through the authenticated thumbnail endpoint', async () => {
    const signal = new AbortController().signal;
    const blob = new Blob(['thumbnail'], { type: 'image/webp' });
    vi.mocked(axios.get).mockResolvedValueOnce({ data: blob } as never);

    await expect(getFileThumbnailBlob('file/thumb', signal)).resolves.toBe(blob);
    expect(axios.get).toHaveBeenCalledExactlyOnceWith(fileEndpoints.thumbnail('file/thumb'), {
      responseType: 'blob',
      signal,
    });
  });
});
