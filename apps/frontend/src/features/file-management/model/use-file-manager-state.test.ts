import type { FileEntry } from 'src/entities/file';

import { it, vi, expect, describe } from 'vitest';

import { ROOT_DIRECTORY_ID, DEFAULT_FILE_FILTERS } from './constants';
import {
  fileManagerScopeKey,
  fileManagerParentId,
  changeFileManagerSpace,
  fileManagerSpaceReducer,
  type FileManagerSpaceState,
  type FileManagerSpaceAction,
  createFileManagerSpaceState,
  createFileManagerSpaceStateFromRoute,
} from './use-file-manager-state';

describe('file manager space state', () => {
  it('returns one directory level at a time from a nested trail', () => {
    const root = createFileManagerSpaceState('space-1');
    const first = fileManagerSpaceReducer(root, {
      type: 'enter-directory',
      directoryId: 'folder-a',
    });
    const second = fileManagerSpaceReducer(first, {
      type: 'enter-directory',
      directoryId: 'folder-b',
    });
    const parent = fileManagerSpaceReducer(second, { type: 'leave-directory' });
    const rootAgain = fileManagerSpaceReducer(parent, { type: 'leave-directory' });

    expect(second.directoryTrail).toEqual(['folder-a', 'folder-b']);
    expect(fileManagerParentId(second)).toBe('folder-b');
    expect(parent.directoryTrail).toEqual(['folder-a']);
    expect(fileManagerParentId(parent)).toBe('folder-a');
    expect(rootAgain.directoryTrail).toEqual([]);
    expect(fileManagerParentId(rootAgain)).toBeNull();
  });

  it('replaces a route-hydrated current directory with its resolved ancestor chain', () => {
    const routeState = createFileManagerSpaceStateFromRoute({
      spaceId: 'space-1',
      parentId: 'folder-test-111',
      detailId: null,
    });
    const resolved = fileManagerSpaceReducer(routeState, {
      type: 'replace-directory-trail',
      directoryTrail: ['folder-test', 'folder-test-111'],
    });

    expect(resolved.directoryTrail).toEqual(['folder-test', 'folder-test-111']);
    expect(fileManagerParentId(resolved)).toBe('folder-test-111');
  });

  it('does not create a new state when the resolved directory trail is unchanged', () => {
    const state = {
      ...createFileManagerSpaceState('space-1'),
      directoryTrail: ['folder-test', 'folder-test-111'],
    };

    expect(
      fileManagerSpaceReducer(state, {
        type: 'replace-directory-trail',
        directoryTrail: ['folder-test', 'folder-test-111'],
      })
    ).toBe(state);
  });

  it('hydrates a file-manager route into its space, parent directory, and detail target', () => {
    const nested = createFileManagerSpaceStateFromRoute({
      spaceId: 'space-1',
      parentId: 'folder-1',
      detailId: 'file-1',
    });
    const root = createFileManagerSpaceStateFromRoute({
      spaceId: 'space-1',
      parentId: null,
      detailId: 'file-2',
    });

    expect(nested.spaceId).toBe('space-1');
    expect(nested.directoryTrail).toEqual(['folder-1']);
    expect(fileManagerParentId(nested)).toBe('folder-1');
    expect(nested.detailId).toBe('file-1');
    expect(root.directoryTrail).toEqual([]);
    expect(root.detailId).toBe('file-2');
  });

  it('includes the selected space in the cursor scope', () => {
    const ownScope = scopeKey(undefined);
    const managedScope = scopeKey('space-2');

    expect(managedScope).not.toBe(ownScope);
    expect(managedScope).toContain('space-2');
  });

  it('clears directory, selection targets, and dialogs when the space changes', () => {
    const entry = { id: 'entry-1' } as FileEntry;
    const populated = fileManagerSpaceReducer(createFileManagerSpaceState('space-1'), {
      type: 'update',
      patch: {
        directoryTrail: ['folder-1'],
        detailId: entry.id,
        folderOpen: true,
        folderName: 'Pending folder',
        deleteTarget: entry,
        moveTarget: entry,
        moveDestinationId: 'folder-2',
        batchAction: 'trash',
      },
    });

    expect(
      fileManagerSpaceReducer(populated, { type: 'select-space', spaceId: 'space-2' })
    ).toEqual({
      ...createFileManagerSpaceState('space-2'),
      moveDestinationId: ROOT_DIRECTORY_ID,
    });
  });

  it('resets cursor and table selection when a different space is selected', () => {
    const dispatch = vi.fn();
    const resetTable = vi.fn();

    changeFileManagerSpace({
      currentSpaceId: 'space-1',
      nextSpaceId: 'space-2',
      dispatch,
      resetTable,
    });

    expect(dispatch).toHaveBeenCalledWith({ type: 'select-space', spaceId: 'space-2' });
    expect(resetTable).toHaveBeenCalledTimes(1);
  });

  it('clears the upload queue when the upload dialog closes or opens again', () => {
    const selectedFile = new File(['report'], 'quarterly-report.pdf');
    const state = fileManagerSpaceReducer(
      {
        ...createFileManagerSpaceState('space-1'),
        uploadOpen: true,
        uploadItems: [
          {
            id: 'report',
            file: selectedFile,
            relativePath: selectedFile.name,
            digest: null,
            progress: null,
            status: 'completed',
          },
        ],
      } as FileManagerSpaceState,
      { type: 'close-upload' } as FileManagerSpaceAction
    );

    const reopened = fileManagerSpaceReducer(state, { type: 'open-upload' });

    expect(state).toMatchObject({ uploadOpen: false, uploadItems: [] });
    expect(reopened).toMatchObject({ uploadOpen: true, uploadItems: [] });
  });
});

function scopeKey(spaceId: string | undefined) {
  return fileManagerScopeKey({
    spaceId,
    mode: 'active',
    parentId: null,
    filters: DEFAULT_FILE_FILTERS,
  });
}
