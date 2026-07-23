import axios from 'src/shared/api/http-client';

import { fileEndpoints } from './endpoints';

export async function getFileContentBlob(id: string, signal?: AbortSignal) {
  return getFileBlob(fileEndpoints.content(id), signal);
}

export async function getFilePreviewBlob(id: string, signal?: AbortSignal) {
  return getFileBlob(fileEndpoints.preview(id), signal);
}

export async function getFileThumbnailBlob(id: string, signal?: AbortSignal) {
  return getFileBlob(fileEndpoints.thumbnail(id), signal);
}

async function getFileBlob(endpoint: string, signal?: AbortSignal) {
  const response = await axios.get<Blob>(endpoint, {
    responseType: 'blob',
    ...(signal ? { signal } : {}),
  });
  return response.data;
}
