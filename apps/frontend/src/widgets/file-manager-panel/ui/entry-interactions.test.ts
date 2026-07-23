import type { FileEntry } from 'src/entities/file';
import type { FileActionPermissions } from 'src/features/file-management';

import { vi, it, expect, describe } from 'vitest';

import { activateFileEntry, openFileEntryDetails } from './entry-interactions';

const NO_PERMISSIONS: FileActionPermissions = {
  canList: false,
  canQuery: false,
  canDownload: false,
  canEdit: false,
  canRemove: false,
  canRestore: false,
  canPurge: false,
};

const FILE_ENTRY = { id: 'file-1', type: 'file' } as FileEntry;
const FOLDER_ENTRY = { id: 'folder-1', type: 'folder' } as FileEntry;

describe('file entry interactions', () => {
  it('does not open file details without query permission', () => {
    const controller = interactionController(NO_PERMISSIONS);

    activateFileEntry(FILE_ENTRY, controller);
    openFileEntryDetails(FILE_ENTRY, controller);

    expect(controller.actions.openDetail).not.toHaveBeenCalled();
  });

  it('opens file details when query permission is granted', () => {
    const controller = interactionController({ ...NO_PERMISSIONS, canQuery: true });

    activateFileEntry(FILE_ENTRY, controller);

    expect(controller.actions.openDetail).toHaveBeenCalledOnce();
    expect(controller.actions.openDetail).toHaveBeenCalledWith(FILE_ENTRY);
  });

  it('opens folders with list permission even without query permission', () => {
    const controller = interactionController({ ...NO_PERMISSIONS, canList: true });

    activateFileEntry(FOLDER_ENTRY, controller);

    expect(controller.actions.closeDetail).toHaveBeenCalledOnce();
    expect(controller.actions.openFolder).toHaveBeenCalledWith(FOLDER_ENTRY.id);
    expect(controller.actions.openDetail).not.toHaveBeenCalled();
  });

  it('does not open folders without list permission', () => {
    const controller = interactionController(NO_PERMISSIONS);

    activateFileEntry(FOLDER_ENTRY, controller);

    expect(controller.actions.closeDetail).not.toHaveBeenCalled();
    expect(controller.actions.openFolder).not.toHaveBeenCalled();
  });
});

function interactionController(permissions: FileActionPermissions) {
  return {
    permissions,
    actions: {
      openDetail: vi.fn(),
      closeDetail: vi.fn(),
      openFolder: vi.fn(),
    },
  };
}
