import type { CursorPageResponse } from 'src/shared/api/types';

export type FileEntryType = 'file' | 'folder';

export const FILE_ENTRY_SORT_FIELDS = ['name', 'created_at', 'updated_at'] as const;

export type FileEntrySortField = (typeof FILE_ENTRY_SORT_FIELDS)[number];

export function fileEntrySortField(value: string): FileEntrySortField {
  if (FILE_ENTRY_SORT_FIELDS.includes(value as FileEntrySortField)) {
    return value as FileEntrySortField;
  }
  throw new Error(`Unsupported file entry sort field: ${value}`);
}
export type FileTypeCategory =
  | 'folder'
  | 'image'
  | 'video'
  | 'audio'
  | 'text'
  | 'application'
  | 'other';

export type FilePropertyValue = string | number | boolean | null;

export type FileProperties = Readonly<{
  checksum_sha256?: string | null;
  extension?: string | null;
  mime_type?: string | null;
  created_by?: string | null;
  provider_key?: string | null;
  [key: string]: FilePropertyValue | undefined;
}>;

export type FileEntry = Readonly<{
  id: string;
  space_id: string;
  owner_user_id: string;
  owner_name?: string | null;
  parent_id: string | null;
  name: string;
  type: FileEntryType;
  size_bytes: number;
  mime_type: string | null;
  object_url?: string | null;
  thumbnail_url?: string | null;
  created_at: string;
  updated_at: string;
  trashed_at?: string | null;
  tags: string[];
  properties: FileProperties;
  preview_supported: boolean;
  download_only: boolean;
}>;

export type FileDirectoryTrailEntry = Readonly<{
  id: string;
  parent_id: string | null;
  name: string;
}>;

export type FileListQuery = Readonly<{
  space_id?: string;
  parent_id?: string | null;
  kind?: FileEntryType;
  search?: string;
  extension?: string;
  mime_type?: string;
  tag?: string;
  start_time?: string;
  end_time?: string;
  trashed?: boolean;
  sort_by?: FileEntrySortField;
  sort_order?: 'asc' | 'desc';
}>;

export type FileOverview = Readonly<{
  space_id: string;
  logical_asset_size: number;
  managed_physical_usage: number;
  recycle_bin_size: number;
  temporary_upload_size: number;
  deduplication_savings: number;
  quota_bytes: number;
  quota_reserved_bytes: number;
  type_distribution: ReadonlyArray<
    Readonly<{
      entry_type: FileTypeCategory;
      bytes: number;
      count: number;
    }>
  >;
  recent_entries: FileEntry[];
  recent_folders: FileEntry[];
}>;

export type FileSpace = Readonly<{
  id: string;
  owner_user_id: string;
  owner_name: string;
  department_name?: string | null;
  status: 'active' | 'archived';
  logical_asset_size: number;
  managed_physical_usage: number;
  reserved_bytes: number;
  quota_bytes: number;
  updated_at: string;
}>;

export const FILE_SPACE_SORT_FIELDS = [
  'owner_name',
  'department_name',
  'status',
  'logical_asset_size',
  'reserved_bytes',
  'quota_bytes',
  'updated_at',
] as const;

export type FileSpaceSortField = (typeof FILE_SPACE_SORT_FIELDS)[number];

export function fileSpaceSortField(value: string): FileSpaceSortField {
  if (FILE_SPACE_SORT_FIELDS.includes(value as FileSpaceSortField)) {
    return value as FileSpaceSortField;
  }
  throw new Error(`Unsupported file space sort field: ${value}`);
}

export type FileSpaceListQuery = Readonly<{
  owner_user_id?: string;
  search?: string;
  status?: FileSpace['status'];
  sort_by?: FileSpaceSortField;
  sort_order?: 'asc' | 'desc';
}>;

export type FileSpaceListResponse = CursorPageResponse<FileSpace>;

export type FileProviderCapacity =
  | Readonly<{
      Bounded: Readonly<{
        total_bytes: number;
        available_bytes: number;
      }>;
    }>
  | Readonly<{
      UsageBased: Readonly<{
        used_bytes: number;
      }>;
    }>;

export type FileProviderSummary = Readonly<{
  key: string;
  capacity: FileProviderCapacity;
}>;

export type UpdateFileSpaceInput = Readonly<{
  quota_bytes: number | null;
}>;

export type CreateFolderInput = Readonly<{
  space_id: string;
  parent_id: string | null;
  name: string;
}>;

export type UpdateFileInput = Readonly<{
  name?: string;
  parent_id?: string | null;
  tags?: string[];
}>;

export type UploadSessionState = 'open' | 'completing' | 'completed' | 'aborted' | 'expired';

export type UploadPartReceipt = Readonly<{
  part_number: number;
  size_bytes: number;
  sha256: string;
}>;

export type UploadSessionResponse = Readonly<{
  id: string;
  space_id: string;
  parent_id: string | null;
  file_name: string;
  declared_size_bytes: number;
  declared_sha256: string;
  content_type: string;
  part_size: number;
  state: UploadSessionState;
  parts: ReadonlyArray<UploadPartReceipt>;
}>;

export type BeginUploadSessionResponse =
  | Readonly<{
      status: 'upload_required';
      session: UploadSessionResponse;
    }>
  | Readonly<{
      status: 'completed';
      entry: FileEntry;
    }>;

export type BeginUploadSessionInput = Readonly<{
  space_id: string;
  parent_id: string | null;
  file_name: string;
  declared_size_bytes: number;
  declared_sha256: string;
  content_type: string | null;
}>;

export type CompleteUploadInput = Readonly<{
  parts: ReadonlyArray<UploadPartReceipt>;
}>;
