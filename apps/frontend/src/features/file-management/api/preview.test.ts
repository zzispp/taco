import { it, vi, expect, describe, afterEach, beforeEach } from 'vitest';

const { getFilePreviewBlobMock } = vi.hoisted(() => ({
  getFilePreviewBlobMock: vi.fn(),
}));

vi.mock('src/entities/file', () => ({
  getFilePreviewBlob: getFilePreviewBlobMock,
}));

import {
  openFilePreview,
  invalidateFilePreviews,
  type FilePreviewRuntime,
  FilePreviewWindowClosedError,
  FilePreviewWindowBlockedError,
} from './preview';

describe('authenticated file preview', () => {
  beforeEach(() => vi.clearAllMocks());
  afterEach(() => invalidateFilePreviews(['file/1', 'file-1']));

  it('opens a blank tab before loading and revokes the Blob URL after preview load', async () => {
    const blob = new Blob(['preview'], { type: 'image/png' });
    const target = createPreviewWindow();
    const runtime = createRuntime(target.window);
    getFilePreviewBlobMock.mockResolvedValueOnce(blob);

    await openFilePreview('file/1', runtime);

    expect(runtime.openBlank).toHaveBeenCalledBefore(getFilePreviewBlobMock);
    expect(runtime.createObjectURL).toHaveBeenCalledExactlyOnceWith(blob);
    expect(target.replace).toHaveBeenCalledExactlyOnceWith('blob:preview');
    expect(target.window.opener).toBeNull();
    expect(runtime.revokeObjectURL).not.toHaveBeenCalled();

    target.load();
    expect(runtime.revokeObjectURL).toHaveBeenCalledExactlyOnceWith('blob:preview');
  });

  it('does not request protected content when the browser blocks the preview tab', async () => {
    const runtime = createRuntime(null);

    await expect(openFilePreview('file-1', runtime)).rejects.toBeInstanceOf(
      FilePreviewWindowBlockedError
    );
    expect(getFilePreviewBlobMock).not.toHaveBeenCalled();
    expect(runtime.createObjectURL).not.toHaveBeenCalled();
  });

  it('closes the blank tab when authenticated preview loading fails', async () => {
    const target = createPreviewWindow();
    const runtime = createRuntime(target.window);
    const failure = new Error('preview request failed');
    getFilePreviewBlobMock.mockRejectedValueOnce(failure);

    await expect(openFilePreview('file-1', runtime)).rejects.toBe(failure);
    expect(target.close).toHaveBeenCalledTimes(1);
    expect(runtime.createObjectURL).not.toHaveBeenCalled();
  });

  it('revokes the Blob URL and closes the tab when navigation fails', async () => {
    const target = createPreviewWindow();
    const runtime = createRuntime(target.window);
    const failure = new Error('navigation failed');
    target.replace.mockImplementationOnce(() => {
      throw failure;
    });
    getFilePreviewBlobMock.mockResolvedValueOnce(new Blob(['preview']));

    await expect(openFilePreview('file-1', runtime)).rejects.toBe(failure);
    expect(runtime.revokeObjectURL).toHaveBeenCalledExactlyOnceWith('blob:preview');
    expect(target.close).toHaveBeenCalledTimes(1);
  });

  it('ignores the blank tab load before the Blob navigation finishes', async () => {
    const target = createPreviewWindow(true);
    const runtime = createRuntime(target.window);
    getFilePreviewBlobMock.mockResolvedValueOnce(new Blob(['preview']));

    await openFilePreview('file-1', runtime);

    expect(runtime.revokeObjectURL).not.toHaveBeenCalled();
    target.load();
    expect(runtime.revokeObjectURL).toHaveBeenCalledExactlyOnceWith('blob:preview');
  });

  it('closes every loaded preview for an asset moved into the recycle bin', async () => {
    const first = createPreviewWindow();
    const second = createPreviewWindow();
    const runtime = createRuntime(first.window, second.window);
    getFilePreviewBlobMock.mockResolvedValue(new Blob(['preview']));

    await openFilePreview('file-1', runtime);
    await openFilePreview('file-1', runtime);
    first.load();
    second.load();

    invalidateFilePreviews(['file-1']);

    expect(first.close).toHaveBeenCalledExactlyOnceWith();
    expect(second.close).toHaveBeenCalledExactlyOnceWith();
  });

  it('closes a blank preview if the asset is moved before its content finishes loading', async () => {
    const target = createPreviewWindow();
    const runtime = createRuntime(target.window);
    const pending = createDeferred<Blob>();
    getFilePreviewBlobMock.mockReturnValueOnce(pending.promise);

    const preview = openFilePreview('file-1', runtime);
    invalidateFilePreviews(['file-1']);
    pending.resolve(new Blob(['preview']));

    await expect(preview).rejects.toBeInstanceOf(FilePreviewWindowClosedError);
    expect(target.close).toHaveBeenCalledExactlyOnceWith();
  });
});

function createRuntime(
  ...windows: Array<FilePreviewRuntime['openBlank'] extends () => infer T ? T : never>
) {
  const queuedWindows = [...windows];
  return {
    openBlank: vi.fn(() => queuedWindows.shift() ?? null),
    createObjectURL: vi.fn(() => 'blob:preview'),
    revokeObjectURL: vi.fn(),
  } satisfies FilePreviewRuntime;
}

function createPreviewWindow(loadOnSubscribe = false) {
  let onLoad = () => undefined;
  let closed = false;
  const location = {
    href: 'about:blank',
    replace: vi.fn((url: string) => {
      location.href = url;
    }),
  };
  const close = vi.fn(() => {
    closed = true;
  });
  const window = {
    opener: {} as unknown,
    get closed() {
      return closed;
    },
    close,
    location,
    addEventListener: vi.fn((_event, listener) => {
      onLoad = listener;
      if (loadOnSubscribe) listener();
    }),
    removeEventListener: vi.fn(),
  };
  return { window, replace: location.replace, close, load: () => onLoad() };
}

function createDeferred<T>() {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((next) => {
    resolve = next;
  });
  return { promise, resolve };
}
