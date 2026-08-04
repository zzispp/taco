import type { FileListQuery, FileEntrySortField } from 'src/entities/file';

export type FileManagerMode = 'active' | 'trash';
export type FileBatchAction = 'trash' | 'restore' | 'purge';

export type FileViewMode = 'list' | 'grid';

export const BYTES_PER_GIB = 1024 ** 3;
export const FILE_HASH_CHUNK_BYTES = 4 * 1024 ** 2;
export const ROOT_DIRECTORY_ID = '00000000-0000-0000-0000-000000000000';
export const SELF_SPACE_SELECTION = '__self__';

export type FileFilterDraft = Readonly<{
  search: string;
  extension: string;
  tag: string;
}>;

export const DEFAULT_FILE_FILTERS: FileFilterDraft = {
  search: '',
  extension: '',
  tag: '',
};

export function selectedFileSpaceId(value: string): string | undefined {
  return value === SELF_SPACE_SELECTION ? undefined : value;
}

export function fileQuery(
  options: Readonly<{
    filters: FileFilterDraft;
    mode: FileManagerMode;
    parentId: string | null;
    spaceId?: string;
    sortBy?: FileEntrySortField;
    sortOrder?: 'asc' | 'desc';
  }>
): FileListQuery {
  const { filters, mode, parentId, spaceId, sortBy, sortOrder } = options;
  const parentFilter = fileQueryParentFilter({ filters, mode, parentId });
  return {
    ...fileQuerySpace(spaceId),
    ...fileQueryParent(parentFilter),
    ...fileQueryFilters(filters),
    ...fileQueryMode(mode),
    ...fileQuerySort({ sortBy, sortOrder }),
  };
}

function fileQueryParentFilter({
  filters,
  mode,
  parentId,
}: Pick<Parameters<typeof fileQuery>[0], 'filters' | 'mode' | 'parentId'>) {
  if (filters.search) return undefined;
  return parentId ?? (mode === 'active' ? ROOT_DIRECTORY_ID : undefined);
}

function fileQuerySpace(spaceId: string | undefined) {
  return spaceId ? { space_id: spaceId } : {};
}

function fileQueryParent(parentId: string | undefined) {
  return parentId ? { parent_id: parentId } : {};
}

function fileQueryFilters(filters: FileFilterDraft) {
  return {
    ...(filters.search ? { search: filters.search } : {}),
    ...(filters.extension ? { extension: filters.extension } : {}),
    ...(filters.tag ? { tag: filters.tag } : {}),
  };
}

function fileQueryMode(mode: FileManagerMode) {
  return { trashed: mode === 'trash' };
}

function fileQuerySort({
  sortBy,
  sortOrder,
}: Pick<Parameters<typeof fileQuery>[0], 'sortBy' | 'sortOrder'>) {
  return {
    ...(sortBy ? { sort_by: sortBy } : {}),
    ...(sortOrder ? { sort_order: sortOrder } : {}),
  };
}
