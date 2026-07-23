import type { FileActionPermissions } from './file-action-policy';

import { it, expect, describe } from 'vitest';

import {
  canDeleteFileEntry,
  canPreviewFileEntry,
  canSelectFileEntries,
  canUseFileBatchAction,
  canViewFileEntryDetails,
  resolveFileEntryActivation,
} from './file-action-policy';

const NO_PERMISSIONS: FileActionPermissions = {
  canList: false,
  canQuery: false,
  canDownload: false,
  canEdit: false,
  canRemove: false,
  canRestore: false,
  canPurge: false,
};

describe('file action policy', () => {
  it('only enables moving active entries to trash with remove permission', () => {
    const permissions = { ...NO_PERMISSIONS, canRemove: true };

    expect(canUseFileBatchAction('active', permissions, 'trash')).toBe(true);
    expect(canUseFileBatchAction('trash', permissions, 'trash')).toBe(false);
    expect(canSelectFileEntries('trash', permissions)).toBe(false);
    expect(canDeleteFileEntry('trash', permissions)).toBe(false);
  });

  it('enables restore and purge independently in trash mode', () => {
    const restoreOnly = { ...NO_PERMISSIONS, canRestore: true };
    const purgeOnly = { ...NO_PERMISSIONS, canPurge: true };

    expect(canUseFileBatchAction('trash', restoreOnly, 'restore')).toBe(true);
    expect(canUseFileBatchAction('trash', restoreOnly, 'purge')).toBe(false);
    expect(canUseFileBatchAction('trash', purgeOnly, 'restore')).toBe(false);
    expect(canUseFileBatchAction('trash', purgeOnly, 'purge')).toBe(true);
    expect(canSelectFileEntries('trash', restoreOnly)).toBe(true);
    expect(canSelectFileEntries('trash', purgeOnly)).toBe(true);
    expect(canDeleteFileEntry('trash', purgeOnly)).toBe(true);
  });

  it('never exposes trash lifecycle actions in the wrong mode', () => {
    const permissions = {
      ...NO_PERMISSIONS,
      canRemove: true,
      canRestore: true,
      canPurge: true,
    };

    expect(canUseFileBatchAction('active', permissions, 'restore')).toBe(false);
    expect(canUseFileBatchAction('active', permissions, 'purge')).toBe(false);
    expect(canUseFileBatchAction('trash', permissions, 'trash')).toBe(false);
  });

  it('only previews active previewable files with query permission', () => {
    const permissions = { ...NO_PERMISSIONS, canQuery: true };
    const previewable = { type: 'file', preview_supported: true } as const;

    expect(canPreviewFileEntry('active', permissions, previewable)).toBe(true);
    expect(canPreviewFileEntry('trash', permissions, previewable)).toBe(false);
    expect(canPreviewFileEntry('active', NO_PERMISSIONS, previewable)).toBe(false);
    expect(
      canPreviewFileEntry('active', permissions, { type: 'folder', preview_supported: false })
    ).toBe(false);
  });

  it('separates query-gated file details from list-gated folder navigation', () => {
    const file = { type: 'file' } as const;
    const folder = { type: 'folder' } as const;

    expect(canViewFileEntryDetails(NO_PERMISSIONS)).toBe(false);
    expect(resolveFileEntryActivation(file, NO_PERMISSIONS)).toBeNull();
    expect(resolveFileEntryActivation(folder, NO_PERMISSIONS)).toBeNull();

    expect(resolveFileEntryActivation(file, { ...NO_PERMISSIONS, canQuery: true })).toBe(
      'open-details'
    );
    expect(resolveFileEntryActivation(folder, { ...NO_PERMISSIONS, canList: true })).toBe(
      'open-folder'
    );
  });
});
