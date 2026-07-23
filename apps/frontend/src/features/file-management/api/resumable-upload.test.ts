import { mutate } from 'swr';
import { it, vi, expect, describe, beforeEach } from 'vitest';

import axios from 'src/shared/api/http-client';

import { fileEndpoints } from 'src/entities/file';

import { sha256Text } from '../lib/sha256';
import { uploadFile, UploadCancellationError } from './resumable-upload';
import { clearUploadIntent, uploadIntentFingerprint } from './upload-intent';

vi.mock('swr', () => ({ mutate: vi.fn() }));
vi.mock('src/shared/api/http-client', () => ({
  isNormalizedApiError: (error: unknown) =>
    error instanceof Error &&
    Object.hasOwn(error, 'status') &&
    Object.hasOwn(error, 'code') &&
    Object.hasOwn(error, 'details'),
  default: {
    delete: vi.fn(),
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
  },
}));

const INPUT = { space_id: 'space-1', parent_id: null };
const FILE = new File(['abcde'], 'notes.txt', { type: 'text/plain' });
const PARTS = [part(1, 'ab'), part(2, 'cd'), part(3, 'e')];

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(mutate).mockResolvedValue(undefined);
  clearUploadIntent(uploadFingerprint());
});

describe('resumable file upload', () => {
  it('skips server-known parts and completes with the refreshed part list', async () => {
    const initial = session([PARTS[0]]);
    const completed = session(PARTS);
    vi.mocked(axios.post)
      .mockResolvedValueOnce({ data: { status: 'upload_required', session: initial } } as never)
      .mockResolvedValueOnce({ data: fileEntry() } as never);
    vi.mocked(axios.get)
      .mockResolvedValueOnce({ data: initial } as never)
      .mockResolvedValueOnce({ data: completed } as never);
    vi.mocked(axios.put)
      .mockResolvedValueOnce({ data: PARTS[1] } as never)
      .mockResolvedValueOnce({ data: PARTS[2] } as never);

    const result = await uploadFile(FILE, INPUT);

    expect(result.id).toBe('entry-1');
    expect(axios.put).toHaveBeenCalledTimes(2);
    expect(vi.mocked(axios.put).mock.calls[0][0]).toBe(
      fileEndpoints.uploadSessionPart('session-1', 2)
    );
    expect(vi.mocked(axios.put).mock.calls[0][2]).toEqual(
      expect.objectContaining({
        headers: expect.objectContaining({ 'X-Content-SHA256': PARTS[1].sha256 }),
      })
    );
    expect(vi.mocked(axios.post).mock.calls[1][1]).toEqual({ parts: PARTS });
  });

  it('reports bytes transferred inside a part before its receipt is returned', async () => {
    const progressEvents: Array<{
      phase: string;
      completedBytes: number;
      partNumber?: number;
      partCount?: number;
    }> = [];
    const initial = session([]);
    const completed = session(PARTS);
    let uploadedPartIndex = 0;
    vi.mocked(axios.post)
      .mockResolvedValueOnce({ data: { status: 'upload_required', session: initial } } as never)
      .mockResolvedValueOnce({ data: fileEntry() } as never);
    vi.mocked(axios.get)
      .mockResolvedValueOnce({ data: initial } as never)
      .mockResolvedValueOnce({ data: completed } as never);
    vi.mocked(axios.put).mockImplementation(async (_url, _body, config) => {
      config?.onUploadProgress?.({ loaded: 1 } as never);
      const receipt = PARTS[uploadedPartIndex];
      uploadedPartIndex += 1;
      return { data: receipt } as never;
    });

    await uploadFile(FILE, INPUT, {
      onProgress: (progress) => progressEvents.push(progress),
    });

    expect(progressEvents).toContainEqual(
      expect.objectContaining({
        phase: 'uploading',
        completedBytes: 1,
        partNumber: 1,
        partCount: 3,
      })
    );
  });

  it('cancels the server session when an active upload is aborted', async () => {
    const controller = new AbortController();
    const initial = session([]);
    vi.mocked(axios.post).mockResolvedValueOnce({
      data: { status: 'upload_required', session: initial },
    } as never);
    vi.mocked(axios.get).mockResolvedValueOnce({ data: initial } as never);
    vi.mocked(axios.put).mockImplementationOnce(async (_url, _body, config) => {
      controller.abort();
      throw new Error('request interrupted');
    });
    vi.mocked(axios.delete).mockResolvedValueOnce({ data: null } as never);

    await expect(uploadFile(FILE, INPUT, { signal: controller.signal })).rejects.toThrow(
      'request interrupted'
    );
    expect(axios.delete).toHaveBeenCalledExactlyOnceWith(fileEndpoints.uploadSession('session-1'));
  });

  it('exposes a failed cancellation instead of reporting canceled success', async () => {
    const controller = new AbortController();
    const initial = session([]);
    vi.mocked(axios.post).mockResolvedValueOnce({
      data: { status: 'upload_required', session: initial },
    } as never);
    vi.mocked(axios.get).mockResolvedValueOnce({ data: initial } as never);
    vi.mocked(axios.put).mockImplementationOnce(async () => {
      controller.abort();
      throw new Error('request interrupted');
    });
    vi.mocked(axios.delete).mockRejectedValueOnce(new Error('cancel failed'));

    await expect(uploadFile(FILE, INPUT, { signal: controller.signal })).rejects.toBeInstanceOf(
      UploadCancellationError
    );
  });

  it('returns a visible digest reuse without creating or completing a session', async () => {
    vi.mocked(axios.post).mockResolvedValueOnce({
      data: { status: 'completed', entry: fileEntry() },
    } as never);

    const result = await uploadFile(FILE, INPUT);

    expect(result.id).toBe('entry-1');
    expect(axios.post).toHaveBeenCalledTimes(1);
    expect(axios.get).not.toHaveBeenCalled();
    expect(axios.put).not.toHaveBeenCalled();
    expect(axios.delete).not.toHaveBeenCalled();
    expect(mutate).toHaveBeenCalledTimes(5);
  });

  it('uses a caller-calculated digest without reporting a second hashing phase', async () => {
    const digest = sha256Text('abcde');
    const progress: string[] = [];
    vi.mocked(axios.post).mockResolvedValueOnce({
      data: { status: 'completed', entry: fileEntry() },
    } as never);

    await uploadFile(FILE, INPUT, {
      digest,
      onProgress: (event) => progress.push(event.phase),
    });

    expect(axios.post).toHaveBeenCalledWith(
      fileEndpoints.uploadSessions,
      expect.objectContaining({ declared_sha256: digest }),
      expect.any(Object)
    );
    expect(progress).toEqual(['completing']);
  });

  it('retains the idempotency key until completed resources refresh successfully', async () => {
    vi.mocked(axios.post)
      .mockResolvedValueOnce({ data: { status: 'completed', entry: fileEntry() } } as never)
      .mockResolvedValueOnce({ data: { status: 'completed', entry: fileEntry() } } as never);
    vi.mocked(mutate).mockRejectedValueOnce(new Error('refresh failed'));

    await expect(uploadFile(FILE, INPUT)).rejects.toThrow('refresh failed');
    await uploadFile(FILE, INPUT);

    const firstHeaders = vi.mocked(axios.post).mock.calls[0][2]?.headers;
    const secondHeaders = vi.mocked(axios.post).mock.calls[1][2]?.headers;
    expect(firstHeaders).toEqual(secondHeaders);
  });

  it('rotates a terminal intent key and retries begin exactly once', async () => {
    vi.mocked(axios.post)
      .mockRejectedValueOnce(normalizedError('upload_intent_terminal'))
      .mockResolvedValueOnce({ data: { status: 'completed', entry: fileEntry() } } as never);

    await uploadFile(FILE, INPUT);

    const firstKey = vi.mocked(axios.post).mock.calls[0][2]?.headers?.['Idempotency-Key'];
    const secondKey = vi.mocked(axios.post).mock.calls[1][2]?.headers?.['Idempotency-Key'];
    expect(firstKey).toBeTypeOf('string');
    expect(secondKey).toBeTypeOf('string');
    expect(secondKey).not.toBe(firstKey);
  });

  it('does not rotate or retry non-terminal begin errors', async () => {
    vi.mocked(axios.post)
      .mockRejectedValueOnce(normalizedError('conflict'))
      .mockResolvedValueOnce({ data: { status: 'completed', entry: fileEntry() } } as never);

    await expect(uploadFile(FILE, INPUT)).rejects.toMatchObject({ code: 'conflict' });
    expect(axios.post).toHaveBeenCalledOnce();
    await uploadFile(FILE, INPUT);

    const firstKey = vi.mocked(axios.post).mock.calls[0][2]?.headers?.['Idempotency-Key'];
    const secondKey = vi.mocked(axios.post).mock.calls[1][2]?.headers?.['Idempotency-Key'];
    expect(secondKey).toBe(firstKey);
  });

  it('clears a replacement key when the single retry is also terminal', async () => {
    vi.mocked(axios.post)
      .mockRejectedValueOnce(normalizedError('upload_result_unavailable'))
      .mockRejectedValueOnce(normalizedError('upload_intent_terminal'))
      .mockResolvedValueOnce({ data: { status: 'completed', entry: fileEntry() } } as never);

    await expect(uploadFile(FILE, INPUT)).rejects.toMatchObject({ code: 'upload_intent_terminal' });
    await uploadFile(FILE, INPUT);

    expect(axios.post).toHaveBeenCalledTimes(3);
    const retryKey = vi.mocked(axios.post).mock.calls[1][2]?.headers?.['Idempotency-Key'];
    const nextKey = vi.mocked(axios.post).mock.calls[2][2]?.headers?.['Idempotency-Key'];
    expect(nextKey).not.toBe(retryKey);
  });
});

function uploadFingerprint() {
  return uploadIntentFingerprint({
    spaceId: INPUT.space_id,
    parentId: INPUT.parent_id,
    fileName: FILE.name,
    fileSize: FILE.size,
    digest: sha256Text('abcde'),
    contentType: FILE.type,
  });
}

function normalizedError(code: string) {
  return Object.assign(new Error(code), { status: 409, code, details: code });
}

function session(parts: ReadonlyArray<ReturnType<typeof part>>) {
  return {
    id: 'session-1',
    space_id: INPUT.space_id,
    parent_id: INPUT.parent_id,
    file_name: FILE.name,
    declared_size_bytes: FILE.size,
    declared_sha256: sha256Text('abcde'),
    content_type: FILE.type,
    part_size: 2,
    state: 'open' as const,
    parts: [...parts],
  };
}

function part(partNumber: number, value: string) {
  return {
    part_number: partNumber,
    size_bytes: value.length,
    sha256: sha256Text(value),
  };
}

function fileEntry() {
  return {
    id: 'entry-1',
    space_id: INPUT.space_id,
    owner_user_id: 'user-1',
    owner_name: 'User',
    parent_id: null,
    name: FILE.name,
    type: 'file' as const,
    size_bytes: FILE.size,
    mime_type: FILE.type,
    object_url: null,
    thumbnail_url: null,
    created_at: '2026-07-20T00:00:00Z',
    updated_at: '2026-07-20T00:00:00Z',
    trashed_at: null,
    tags: [],
    properties: {},
    preview_supported: true,
    download_only: false,
  };
}
