import type { FileEntry } from 'src/entities/file';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { FileManagerController } from 'src/features/file-management';

import {
  canEditFileEntry,
  canDeleteFileEntry,
  canPreviewFileEntry,
  canRestoreFileEntry,
  canDownloadFileEntry,
  canViewFileEntryDetails,
} from 'src/features/file-management';

type Translate = (key: string) => string;

type EntryItemsOptions = Readonly<{
  entry: FileEntry;
  controller: FileManagerController;
  t: Translate;
  capabilities: ReturnType<typeof entryMenuCapabilities>;
}>;

export type FileManagerContextMenuItem = Readonly<{
  id: string;
  icon: IconifyName;
  label: string;
  onClick: () => void;
  destructive?: boolean;
}>;

export type FileManagerContextMenuContent = FileManagerContextMenuItem | 'divider';

export function hasDirectoryMenuActions(controller: FileManagerController) {
  const capabilities = directoryMenuCapabilities(controller);
  return capabilities.canGoToParent || capabilities.canCreateFolder || capabilities.canUpload;
}

export function hasEntryMenuActions(entry: FileEntry, controller: FileManagerController) {
  return Object.values(entryMenuCapabilities(entry, controller)).some(Boolean);
}

export function directoryMenuContent(
  controller: FileManagerController,
  t: Translate
): FileManagerContextMenuContent[] {
  const { canGoToParent, canCreateFolder, canUpload } = directoryMenuCapabilities(controller);
  const creationItems = withoutNullMenuItems([
    canCreateFolder
      ? {
          id: 'new-folder',
          icon: 'solar:add-folder-bold' as const,
          label: t('file.actions.newFolder'),
          onClick: controller.actions.openFolderDialog,
        }
      : null,
    canUpload
      ? {
          id: 'upload',
          icon: 'eva:cloud-upload-fill' as const,
          label: t('file.actions.upload'),
          onClick: controller.actions.openUpload,
        }
      : null,
  ]);
  const parentItem: FileManagerContextMenuItem | null = canGoToParent
    ? {
        id: 'up-one-level',
        icon: 'eva:arrow-ios-back-fill' as const,
        label: t('file.actions.upOneLevel'),
        onClick: controller.actions.goToParentFolder,
      }
    : null;

  return [
    ...(parentItem ? [parentItem] : []),
    ...(parentItem && creationItems.length ? ['divider' as const] : []),
    ...creationItems,
  ];
}

export function entryMenuContent(
  entry: FileEntry,
  controller: FileManagerController,
  t: Translate
): FileManagerContextMenuContent[] {
  const capabilities = entryMenuCapabilities(entry, controller);
  const options = { entry, controller, t, capabilities };
  const primaryItems = entryPrimaryItems(options);
  const lifecycleItems = entryLifecycleItems(options);
  return [
    ...primaryItems,
    ...(lifecycleItems.length ? ['divider' as const] : []),
    ...lifecycleItems,
  ];
}

function directoryMenuCapabilities(controller: FileManagerController) {
  const activeDirectory = controller.state.mode !== 'trash';
  return {
    canGoToParent: Boolean(
      activeDirectory && controller.state.parentId && controller.resources.directoryTrail.length
    ),
    canCreateFolder: activeDirectory && controller.permissions.canAddFolder,
    canUpload: activeDirectory && controller.permissions.canUpload,
  };
}

function entryMenuCapabilities(entry: FileEntry, controller: FileManagerController) {
  const activeDirectory = controller.state.mode !== 'trash';
  return {
    canOpenFolder: entry.type === 'folder' && activeDirectory && controller.permissions.canList,
    canPreview: canPreviewFileEntry(controller.state.mode, controller.permissions, entry),
    canDownload:
      entry.type === 'file' && canDownloadFileEntry(controller.state.mode, controller.permissions),
    canViewDetails: canViewFileEntryDetails(controller.permissions),
    canEdit: canEditFileEntry(controller.state.mode, controller.permissions),
    canRestore: canRestoreFileEntry(controller.state.mode, controller.permissions),
    canDelete: canDeleteFileEntry(controller.state.mode, controller.permissions),
  };
}

function entryPrimaryItems({
  entry,
  controller,
  t,
  capabilities,
}: EntryItemsOptions): FileManagerContextMenuItem[] {
  return withoutNullMenuItems([
    capabilities.canOpenFolder
      ? {
          id: 'open',
          icon: 'eva:arrow-ios-forward-fill' as const,
          label: t('file.actions.open'),
          onClick: () => openFolder(entry, controller),
        }
      : null,
    capabilities.canPreview
      ? {
          id: 'preview',
          icon: 'solar:eye-bold' as const,
          label: t('file.preview'),
          onClick: () => controller.actions.previewEntry(entry),
        }
      : null,
    capabilities.canDownload
      ? {
          id: 'download',
          icon: 'solar:download-bold' as const,
          label: t('file.actions.download'),
          onClick: () => controller.actions.downloadEntry(entry),
        }
      : null,
    capabilities.canViewDetails
      ? {
          id: 'details',
          icon: 'solar:info-circle-bold' as const,
          label: t('file.info'),
          onClick: () => controller.actions.openDetail(entry),
        }
      : null,
    capabilities.canEdit
      ? {
          id: 'move',
          icon: 'eva:arrow-forward-fill' as const,
          label: t('file.actions.move'),
          onClick: () => controller.actions.requestMove(entry),
        }
      : null,
  ]);
}

function entryLifecycleItems({
  entry,
  controller,
  t,
  capabilities,
}: EntryItemsOptions): FileManagerContextMenuItem[] {
  return withoutNullMenuItems([
    capabilities.canRestore
      ? {
          id: 'restore',
          icon: 'solar:restart-bold' as const,
          label: t('file.actions.restore'),
          onClick: () => controller.actions.restoreEntry(entry),
        }
      : null,
    capabilities.canDelete
      ? {
          id: 'delete',
          icon: 'solar:trash-bin-trash-bold' as const,
          label: t(
            controller.state.mode === 'trash' ? 'file.actions.purge' : 'file.actions.delete'
          ),
          onClick: () => controller.actions.requestDelete(entry),
          destructive: true,
        }
      : null,
  ]);
}

function withoutNullMenuItems(
  items: ReadonlyArray<FileManagerContextMenuItem | null>
): FileManagerContextMenuItem[] {
  return items.filter((item): item is FileManagerContextMenuItem => item !== null);
}

function openFolder(entry: FileEntry, controller: FileManagerController) {
  controller.actions.closeDetail();
  controller.actions.openFolder(entry.id);
}
