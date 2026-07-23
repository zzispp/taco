import type { UploadInput, UploadOptions } from './resumable-upload-contract';
import type {
  FileEntry,
  UploadPartReceipt,
  CompleteUploadInput,
  UploadSessionResponse,
  BeginUploadSessionInput,
  BeginUploadSessionResponse,
} from 'src/entities/file';

import axios from 'src/shared/api/http-client';
import { requestData } from 'src/shared/api/pagination';

import { fileEndpoints } from 'src/entities/file';

import { sha256Blob } from '../lib/sha256';
import { validateReceipt } from './upload-parts';

const PART_DIGEST_HEADER = 'X-Content-SHA256';

export async function createUploadSession(
  config: Readonly<{
    file: File;
    input: UploadInput;
    digest: string;
    idempotencyKey: string;
    signal?: AbortSignal;
  }>
): Promise<BeginUploadSessionResponse> {
  const payload: BeginUploadSessionInput = {
    ...config.input,
    file_name: config.file.name,
    declared_size_bytes: config.file.size,
    declared_sha256: config.digest,
    content_type: config.file.type || null,
  };
  return requestData(
    axios.post<BeginUploadSessionResponse>(fileEndpoints.uploadSessions, payload, {
      signal: config.signal,
      headers: { 'Idempotency-Key': config.idempotencyKey },
    })
  );
}

export async function getUploadSession(id: string, signal?: AbortSignal) {
  return requestData(axios.get<UploadSessionResponse>(fileEndpoints.uploadSession(id), { signal }));
}

export async function uploadPart(
  config: Readonly<{
    file: File;
    session: UploadSessionResponse;
    partNumber: number;
    expectedSize: number;
    options: UploadOptions;
    onTransferProgress?: (completedBytes: number) => void;
  }>
): Promise<UploadPartReceipt> {
  const start = (config.partNumber - 1) * config.session.part_size;
  const blob = config.file.slice(start, start + config.expectedSize);
  if (blob.size !== config.expectedSize) {
    throw new Error(`Upload part ${config.partNumber} has an unexpected size`);
  }
  const digest = await sha256Blob(blob, config.options.signal);
  const receipt = await requestData(
    axios.put<UploadPartReceipt>(
      fileEndpoints.uploadSessionPart(config.session.id, config.partNumber),
      blob,
      {
        signal: config.options.signal,
        headers: { 'Content-Type': 'application/octet-stream', [PART_DIGEST_HEADER]: digest },
        onUploadProgress: (event) =>
          config.onTransferProgress?.(Math.min(event.loaded, config.expectedSize)),
      }
    )
  );
  validateReceipt({
    receipt,
    partNumber: config.partNumber,
    expectedSize: config.expectedSize,
    digest,
  });
  return receipt;
}

export async function completeUpload(
  id: string,
  payload: CompleteUploadInput,
  signal?: AbortSignal
): Promise<FileEntry> {
  return requestData(
    axios.post<FileEntry>(fileEndpoints.uploadSessionComplete(id), payload, { signal })
  );
}

export async function cancelUploadSession(id: string): Promise<void> {
  await axios.delete(fileEndpoints.uploadSession(id));
}
