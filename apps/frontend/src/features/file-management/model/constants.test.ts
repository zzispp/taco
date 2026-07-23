import { it, expect, describe } from 'vitest';

import {
  fileQuery,
  ROOT_DIRECTORY_ID,
  selectedFileSpaceId,
  DEFAULT_FILE_FILTERS,
  SELF_SPACE_SELECTION,
} from './constants';

describe('file manager query', () => {
  it('sends the backend root sentinel for the active root directory', () => {
    expect(
      fileQuery({
        filters: DEFAULT_FILE_FILTERS,
        mode: 'active',
        parentId: null,
        sortBy: 'updated_at',
        sortOrder: 'desc',
      })
    ).toEqual({
      parent_id: ROOT_DIRECTORY_ID,
      trashed: false,
      sort_by: 'updated_at',
      sort_order: 'desc',
    });
  });

  it('sends the selected server-side sort with the file query', () => {
    expect(
      fileQuery({
        filters: DEFAULT_FILE_FILTERS,
        mode: 'active',
        parentId: 'folder-1',
        sortBy: 'name',
        sortOrder: 'asc',
      })
    ).toMatchObject({ sort_by: 'name', sort_order: 'asc' });
  });

  it('searches the full current asset space instead of only the current directory', () => {
    expect(
      fileQuery({
        filters: { ...DEFAULT_FILE_FILTERS, search: 'zwj.yaml' },
        mode: 'active',
        parentId: null,
        spaceId: 'space-1',
      })
    ).toEqual({
      space_id: 'space-1',
      search: 'zwj.yaml',
      trashed: false,
    });
  });

  it('uses a concrete folder id when navigating into a directory', () => {
    expect(
      fileQuery({ filters: DEFAULT_FILE_FILTERS, mode: 'active', parentId: 'folder-1' })
    ).toEqual({ parent_id: 'folder-1', trashed: false });
  });

  it('restores the current-directory constraint after clearing a search', () => {
    expect(
      fileQuery({
        filters: { ...DEFAULT_FILE_FILTERS, search: 'zwj.yaml' },
        mode: 'active',
        parentId: 'folder-1',
      })
    ).toEqual({ search: 'zwj.yaml', trashed: false });

    expect(
      fileQuery({ filters: DEFAULT_FILE_FILTERS, mode: 'active', parentId: 'folder-1' })
    ).toEqual({ parent_id: 'folder-1', trashed: false });
  });

  it('keeps trash as a cross-directory collection', () => {
    expect(fileQuery({ filters: DEFAULT_FILE_FILTERS, mode: 'trash', parentId: null })).toEqual({
      trashed: true,
    });
  });

  it('maps the non-empty own-space select sentinel back to an implicit space', () => {
    expect(selectedFileSpaceId(SELF_SPACE_SELECTION)).toBeUndefined();
    expect(selectedFileSpaceId('managed-space')).toBe('managed-space');
  });
});
