import type { FileEntry } from 'src/entities/file';
import type { FileActionPermissions, FileManagerController } from 'src/features/file-management';

import { canViewFileEntryDetails, resolveFileEntryActivation } from 'src/features/file-management';

type EntryInteractionController = Readonly<{
  permissions: FileActionPermissions;
  actions: Pick<FileManagerController['actions'], 'openDetail' | 'closeDetail' | 'openFolder'>;
}>;

export function activateFileEntry(entry: FileEntry, controller: EntryInteractionController) {
  const activation = resolveFileEntryActivation(entry, controller.permissions);
  if (activation === 'open-folder') {
    controller.actions.closeDetail();
    controller.actions.openFolder(entry.id);
    return;
  }
  if (activation === 'open-details') controller.actions.openDetail(entry);
}

export function openFileEntryDetails(entry: FileEntry, controller: EntryInteractionController) {
  if (entry.type !== 'file' || !canViewFileEntryDetails(controller.permissions)) return;
  controller.actions.openDetail(entry);
}
