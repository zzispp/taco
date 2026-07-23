import type { FileEntry } from 'src/entities/file';

import { it, expect, describe } from 'vitest';

import {
  fileManagerRouteKey,
  parseFileManagerRoute,
  EMPTY_FILE_MANAGER_ROUTE,
  buildFileManagerEntryPath,
} from './file-manager-route';

describe('file manager route contract', () => {
  it('builds a folder destination inside its own directory', () => {
    expect(
      buildFileManagerEntryPath(entry({ id: 'folder-1', parent_id: 'parent-1', type: 'folder' }))
    ).toBe('/dashboard/file-manager?space_id=space-1&parent_id=folder-1');
  });

  it('builds a nested file destination with its details selected', () => {
    expect(buildFileManagerEntryPath(entry({ id: 'file-1', parent_id: 'folder-1' }))).toBe(
      '/dashboard/file-manager?space_id=space-1&parent_id=folder-1&detail_id=file-1'
    );
  });

  it('omits parent_id for a root file', () => {
    expect(buildFileManagerEntryPath(entry({ id: 'file-1', parent_id: null }))).toBe(
      '/dashboard/file-manager?space_id=space-1&detail_id=file-1'
    );
  });

  it('parses meaningful query values and drops blank values', () => {
    expect(
      parseFileManagerRoute(
        new URLSearchParams('space_id=%20space-1%20&parent_id=%20&detail_id=%20file-1%20')
      )
    ).toEqual({ spaceId: 'space-1', parentId: null, detailId: 'file-1' });
    expect(parseFileManagerRoute(new URLSearchParams())).toEqual(EMPTY_FILE_MANAGER_ROUTE);
  });

  it('creates a stable route key from the normalized route shape', () => {
    expect(fileManagerRouteKey({ spaceId: 'space-1', parentId: null, detailId: 'file-1' })).toBe(
      '["space-1",null,"file-1"]'
    );
    expect(fileManagerRouteKey(EMPTY_FILE_MANAGER_ROUTE)).toBe('[null,null,null]');
  });
});

function entry(
  overrides: Partial<Pick<FileEntry, 'id' | 'space_id' | 'parent_id' | 'type'>>
): Pick<FileEntry, 'id' | 'space_id' | 'parent_id' | 'type'> {
  return {
    id: 'file-1',
    space_id: 'space-1',
    parent_id: null,
    type: 'file',
    ...overrides,
  };
}
