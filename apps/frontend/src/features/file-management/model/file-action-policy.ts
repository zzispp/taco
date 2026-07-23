import type { FileEntry } from 'src/entities/file';
import type { fileCapabilities } from './permissions';
import type { FileBatchAction, FileManagerMode } from './constants';

export type FileActionPermissions = Pick<
  ReturnType<typeof fileCapabilities>,
  'canList' | 'canQuery' | 'canDownload' | 'canEdit' | 'canRemove' | 'canRestore' | 'canPurge'
>;

export type FileEntryActivation = 'open-folder' | 'open-details';

export function canViewFileEntryDetails(permissions: FileActionPermissions): boolean {
  return permissions.canQuery;
}

export function resolveFileEntryActivation(
  entry: Pick<FileEntry, 'type'>,
  permissions: FileActionPermissions
): FileEntryActivation | null {
  if (entry.type === 'folder') return permissions.canList ? 'open-folder' : null;
  return canViewFileEntryDetails(permissions) ? 'open-details' : null;
}

export function canUseFileBatchAction(
  mode: FileManagerMode,
  permissions: FileActionPermissions,
  action: FileBatchAction
): boolean {
  if (action === 'trash') return mode !== 'trash' && permissions.canRemove;
  if (action === 'restore') return mode === 'trash' && permissions.canRestore;
  return mode === 'trash' && permissions.canPurge;
}

export function canSelectFileEntries(
  mode: FileManagerMode,
  permissions: FileActionPermissions
): boolean {
  return (['trash', 'restore', 'purge'] as const).some((action) =>
    canUseFileBatchAction(mode, permissions, action)
  );
}

export function canEditFileEntry(
  mode: FileManagerMode,
  permissions: FileActionPermissions
): boolean {
  return mode !== 'trash' && permissions.canEdit;
}

export function canDownloadFileEntry(
  mode: FileManagerMode,
  permissions: FileActionPermissions
): boolean {
  return mode !== 'trash' && permissions.canDownload;
}

export function canPreviewFileEntry(
  mode: FileManagerMode,
  permissions: FileActionPermissions,
  entry: Pick<FileEntry, 'type' | 'preview_supported'>
): boolean {
  return (
    mode !== 'trash' && permissions.canQuery && entry.type === 'file' && entry.preview_supported
  );
}

export function canDeleteFileEntry(
  mode: FileManagerMode,
  permissions: FileActionPermissions
): boolean {
  return mode === 'trash' ? permissions.canPurge : permissions.canRemove;
}

export function canRestoreFileEntry(
  mode: FileManagerMode,
  permissions: FileActionPermissions
): boolean {
  return mode === 'trash' && permissions.canRestore;
}
