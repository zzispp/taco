import type {
  FileEntry,
  FileSpace,
  UpdateFileInput,
  CreateFolderInput,
  UpdateFileSpaceInput,
} from 'src/entities/file';

import axios from 'src/shared/api/http-client';
import { requestData } from 'src/shared/api/pagination';
import { downloadBlobResponse } from 'src/shared/api/download';

import { fileEndpoints } from 'src/entities/file';

import { refreshFileResources } from './refresh';
import { invalidateFilePreviews } from './preview';

export { openFilePreview } from './preview';

export async function createFolder(
  input: CreateFolderInput,
  options: Readonly<{ refresh?: boolean }> = {}
) {
  const folder = await requestData<FileEntry>(axios.post(fileEndpoints.folders, input));
  if (options.refresh !== false) await refreshFileResources();
  return folder;
}

export async function updateFile(id: string, input: UpdateFileInput) {
  const file = await requestData<FileEntry>(axios.put(fileEndpoints.asset(id), input));
  await refreshFileResources();
  return file;
}

export async function moveFile(id: string, parentId: string | null) {
  return updateFile(id, { parent_id: parentId });
}

export async function trashFile(id: string) {
  await axios.post(fileEndpoints.trash(id));
  invalidateFilePreviews([id]);
  await refreshFileResources();
}

export async function restoreFile(id: string) {
  await axios.post(fileEndpoints.restore(id));
  await refreshFileResources();
}

export async function permanentlyDeleteFile(id: string) {
  await axios.delete(fileEndpoints.purge(id));
  invalidateFilePreviews([id]);
  await refreshFileResources();
}

export async function trashFiles(ids: string[]) {
  await axios.post(fileEndpoints.batchTrash, { ids });
  invalidateFilePreviews(ids);
  await refreshFileResources();
}

export async function restoreFiles(ids: string[]) {
  await axios.post(fileEndpoints.batchRestore, { ids });
  await refreshFileResources();
}

export async function permanentlyDeleteFiles(ids: string[]) {
  await axios.post(fileEndpoints.batchPurge, { ids });
  invalidateFilePreviews(ids);
  await refreshFileResources();
}

export async function updateFileSpace(id: string, input: UpdateFileSpaceInput) {
  const space = await requestData<FileSpace>(axios.put(fileEndpoints.space(id), input));
  await refreshFileResources();
  return space;
}

export { uploadFile, cancelUploadSession, UploadCancellationError } from './resumable-upload';

export async function downloadFile(entry: Pick<FileEntry, 'id' | 'name'>) {
  const response = await axios.get<Blob>(fileEndpoints.content(entry.id), { responseType: 'blob' });
  downloadBlobResponse(response, entry.name);
}
