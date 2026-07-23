import type { UploadInput, UploadOptions } from './resumable-upload-contract';
import type {
  FileEntry,
  UploadSessionResponse,
  BeginUploadSessionResponse,
} from 'src/entities/file';

import { isNormalizedApiError } from 'src/shared/api/http-client';

import { sha256File } from '../lib/sha256';
import { refreshFileResources } from './refresh';
import { FILE_HASH_CHUNK_BYTES } from '../model/constants';
import { partLength, receiptMap, completeParts, uploadedBytes } from './upload-parts';
import { uploadIntentKey, clearUploadIntent, uploadIntentFingerprint } from './upload-intent';
import {
  uploadPart,
  completeUpload,
  getUploadSession,
  createUploadSession,
  cancelUploadSession,
} from './resumable-upload-transport';

const TERMINAL_UPLOAD_INTENT_CODES = new Set([
  'upload_intent_terminal',
  'upload_result_unavailable',
]);

type UploadMissingPartsConfig = Readonly<{
  file: File;
  session: UploadSessionResponse;
  options: UploadOptions;
}>;

class TerminalUploadSessionError extends Error {}

export class UploadCancellationError extends Error {
  constructor(cause: unknown) {
    super('Upload session cancellation failed', { cause });
    this.name = 'UploadCancellationError';
  }
}

export async function uploadFile(
  file: File,
  input: UploadInput,
  options: UploadOptions = {}
): Promise<FileEntry> {
  const digest = options.digest ?? (await calculateUploadDigest(file, options));
  const intentFingerprint = uploadIntentFingerprint({
    spaceId: input.space_id,
    parentId: input.parent_id,
    fileName: file.name,
    fileSize: file.size,
    digest,
    contentType: file.type || 'application/octet-stream',
  });
  const created = await createUploadSessionWithRetry({
    file,
    input,
    digest,
    intentFingerprint,
    signal: options.signal,
  });
  return resolveCreatedUpload({ created, file, input, digest, intentFingerprint, options });
}

async function calculateUploadDigest(file: File, options: UploadOptions): Promise<string> {
  return sha256File(file, {
    chunkSize: FILE_HASH_CHUNK_BYTES,
    signal: options.signal,
    onProgress: (completedBytes) =>
      options.onProgress?.({ phase: 'hashing', completedBytes, totalBytes: file.size }),
  });
}

async function createUploadSessionWithRetry(
  config: Readonly<{
    file: File;
    input: UploadInput;
    digest: string;
    intentFingerprint: string;
    signal?: AbortSignal;
  }>
): Promise<BeginUploadSessionResponse> {
  try {
    return await createUploadSession({
      ...config,
      idempotencyKey: uploadIntentKey(config.intentFingerprint),
    });
  } catch (error) {
    if (!isTerminalUploadIntentError(error)) throw error;
  }
  clearUploadIntent(config.intentFingerprint);
  try {
    return await createUploadSession({
      ...config,
      idempotencyKey: uploadIntentKey(config.intentFingerprint),
    });
  } catch (error) {
    if (isTerminalUploadIntentError(error)) clearUploadIntent(config.intentFingerprint);
    throw error;
  }
}

function isTerminalUploadIntentError(error: unknown): boolean {
  return isNormalizedApiError(error) && TERMINAL_UPLOAD_INTENT_CODES.has(error.code ?? '');
}

async function resolveCreatedUpload(
  config: Readonly<{
    created: BeginUploadSessionResponse;
    file: File;
    input: UploadInput;
    digest: string;
    intentFingerprint: string;
    options: UploadOptions;
  }>
): Promise<FileEntry> {
  if (config.created.status === 'completed') {
    config.options.onProgress?.({
      phase: 'completing',
      completedBytes: config.file.size,
      totalBytes: config.file.size,
    });
    await refreshFileResources();
    clearUploadIntent(config.intentFingerprint);
    return config.created.entry;
  }
  try {
    const entry = await resumeUpload({
      file: config.file,
      input: config.input,
      digest: config.digest,
      sessionId: config.created.session.id,
      options: config.options,
    });
    await refreshFileResources();
    clearUploadIntent(config.intentFingerprint);
    return entry;
  } catch (error) {
    if (config.options.signal?.aborted) {
      try {
        await cancelUploadSession(config.created.session.id);
      } catch (cause) {
        throw new UploadCancellationError(cause);
      }
      clearUploadIntent(config.intentFingerprint);
    } else if (error instanceof TerminalUploadSessionError) {
      clearUploadIntent(config.intentFingerprint);
    }
    throw error;
  }
}

async function resumeUpload(
  config: Readonly<{
    file: File;
    input: UploadInput;
    digest: string;
    sessionId: string;
    options: UploadOptions;
  }>
): Promise<FileEntry> {
  let session = await getUploadSession(config.sessionId, config.options.signal);
  assertSessionMatches({ session, file: config.file, input: config.input, digest: config.digest });
  if (session.state === 'aborted' || session.state === 'expired') {
    throw new TerminalUploadSessionError(`Upload session is ${session.state}`);
  }
  const partCount = await uploadMissingParts({
    file: config.file,
    session,
    options: config.options,
  });
  session = await getUploadSession(session.id, config.options.signal);
  const parts = completeParts({
    receipts: session.parts,
    fileSize: config.file.size,
    partSize: session.part_size,
  });
  config.options.onProgress?.({
    phase: 'completing',
    completedBytes: config.file.size,
    totalBytes: config.file.size,
    partCount,
  });
  return completeUpload(session.id, { parts }, config.options.signal);
}

async function uploadMissingParts(config: UploadMissingPartsConfig): Promise<number> {
  const receipts = receiptMap(config.session.parts, config.file.size, config.session.part_size);
  const partCount = Math.ceil(config.file.size / config.session.part_size);
  let completedBytes = uploadedBytes(receipts.values());
  reportUploadingProgress({
    options: config.options,
    completedBytes,
    totalBytes: config.file.size,
    partCount,
  });
  for (
    let partNumber = 1;
    config.session.state === 'open' && partNumber <= partCount;
    partNumber += 1
  ) {
    if (receipts.has(partNumber)) continue;
    const expectedSize = partLength(config.file.size, config.session.part_size, partNumber);
    const completedBeforePart = completedBytes;
    const receipt = await uploadPart({
      file: config.file,
      session: config.session,
      partNumber,
      expectedSize,
      options: config.options,
      onTransferProgress: (partCompletedBytes) =>
        reportUploadingProgress({
          options: config.options,
          completedBytes: completedBeforePart + partCompletedBytes,
          totalBytes: config.file.size,
          partNumber,
          partCount,
        }),
    });
    receipts.set(partNumber, receipt);
    completedBytes += receipt.size_bytes;
    reportUploadingProgress({
      options: config.options,
      completedBytes,
      totalBytes: config.file.size,
      partNumber,
      partCount,
    });
  }
  return partCount;
}

function reportUploadingProgress(
  config: Readonly<{
    options: UploadOptions;
    completedBytes: number;
    totalBytes: number;
    partCount: number;
    partNumber?: number;
  }>
) {
  config.options.onProgress?.({
    phase: 'uploading',
    completedBytes: config.completedBytes,
    totalBytes: config.totalBytes,
    partCount: config.partCount,
    ...(config.partNumber === undefined ? {} : { partNumber: config.partNumber }),
  });
}

function assertSessionMatches(
  config: Readonly<{
    session: UploadSessionResponse;
    file: File;
    input: UploadInput;
    digest: string;
  }>
) {
  if (
    config.session.space_id !== config.input.space_id ||
    config.session.parent_id !== config.input.parent_id ||
    config.session.file_name !== config.file.name ||
    config.session.declared_size_bytes !== config.file.size ||
    config.session.declared_sha256 !== config.digest ||
    config.session.content_type !== (config.file.type || 'application/octet-stream') ||
    !Number.isSafeInteger(config.session.part_size) ||
    config.session.part_size <= 0
  ) {
    throw new Error('Upload session does not match the selected file');
  }
}

export { cancelUploadSession };
