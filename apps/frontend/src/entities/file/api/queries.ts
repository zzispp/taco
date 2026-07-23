import type { CursorPageRequest } from 'src/shared/api/pagination';
import type {
  FileEntry,
  FileSpace,
  FileOverview,
  FileListQuery,
  FileSpaceListQuery,
  FileProviderSummary,
  FileDirectoryTrailEntry,
} from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { fileEndpoints } from './endpoints';

const FILE_DIRECTORY_TRAIL_CACHE_KEY = 'file-directory-trail';

type FileDirectoryTrailKey = readonly [typeof FILE_DIRECTORY_TRAIL_CACHE_KEY, string];

export function isFileDirectoryTrailKey(key: unknown): key is FileDirectoryTrailKey {
  return (
    Array.isArray(key) &&
    key.length === 2 &&
    key[0] === FILE_DIRECTORY_TRAIL_CACHE_KEY &&
    typeof key[1] === 'string'
  );
}

export function useFileEntries(
  request: CursorPageRequest,
  query: FileListQuery = {},
  enabled = true
) {
  return useCursorResource<FileEntry>({
    endpoint: enabled ? fileEndpoints.assets : '',
    request,
    params: query,
    context: JSON.stringify(query),
  });
}

export function useFileOverview(spaceId?: string, enabled = true) {
  return useSWR<FileOverview>(enabled ? fileEndpoints.overview(spaceId) : null, fetcher, {
    revalidateOnFocus: false,
  });
}

export function useFileProviders(enabled = true) {
  return useSWR<ReadonlyArray<FileProviderSummary>>(
    enabled ? fileEndpoints.providers : null,
    fetcher,
    { revalidateOnFocus: false }
  );
}

export function useFileSpaces(
  request: CursorPageRequest,
  query: FileSpaceListQuery = {},
  enabled = true
) {
  return useCursorResource<FileSpace>({
    endpoint: enabled ? fileEndpoints.spaces : '',
    request,
    params: query,
    context: JSON.stringify(query),
  });
}

export function useFileEntry(id: string | null, enabled = true) {
  const key = id && enabled ? fileEndpoints.asset(id) : null;
  return useSWR<FileEntry>(key, fetcher, { revalidateOnFocus: false });
}

export function useFileDirectoryTrail(directoryId: string | null, enabled = true) {
  const key = fileDirectoryTrailKey(directoryId, enabled);
  return useSWR<readonly FileDirectoryTrailEntry[], Error, FileDirectoryTrailKey | null>(
    key,
    loadFileDirectoryTrail,
    { revalidateOnFocus: false }
  );
}

function fileDirectoryTrailKey(
  directoryId: string | null,
  enabled: boolean
): FileDirectoryTrailKey | null {
  if (!directoryId || !enabled) return null;
  return [FILE_DIRECTORY_TRAIL_CACHE_KEY, directoryId];
}

function loadFileDirectoryTrail([, directoryId]: FileDirectoryTrailKey) {
  return fetcher<readonly FileDirectoryTrailEntry[]>(fileEndpoints.directoryTrail(directoryId));
}
