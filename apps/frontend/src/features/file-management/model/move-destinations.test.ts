import { it, expect, describe } from 'vitest';

import { ROOT_DIRECTORY_ID } from './constants';
import { canMoveToDestination, buildMoveDirectoryBrowser } from './move-destinations';

const rootFolders = [folder('root-a', null, 'A'), folder('root-b', null, 'B')];

describe('file move destinations', () => {
  it('projects only the server-loaded child-folder page for the current directory', () => {
    const browser = buildMoveDirectoryBrowser({
      currentId: ROOT_DIRECTORY_ID,
      trail: [],
      children: rootFolders,
      target: file('file-1', 'root-a'),
    });

    expect(browser).toMatchObject({ currentId: ROOT_DIRECTORY_ID, parentId: ROOT_DIRECTORY_ID });
    expect(browser?.children.map((entry) => entry.id)).toEqual(['root-a', 'root-b']);
  });

  it('hides the moving folder as the only entry into its descendant branch', () => {
    const target = folder('root-a', null, 'A');
    const root = buildMoveDirectoryBrowser({
      currentId: ROOT_DIRECTORY_ID,
      trail: [],
      children: rootFolders,
      target,
    });
    const source = buildMoveDirectoryBrowser({
      currentId: 'root-a',
      trail: [{ id: 'root-a', parent_id: null, name: 'A' }],
      children: [folder('child-a', 'root-a', 'A child')],
      target,
    });

    expect(root?.children.map((entry) => entry.id)).toEqual(['root-b']);
    expect(source).toBeNull();
  });

  it('uses the resolved trail for parent navigation without loading ancestors into the folder page', () => {
    const browser = buildMoveDirectoryBrowser({
      currentId: 'child-a',
      trail: [
        { id: 'root-a', parent_id: null, name: 'A' },
        { id: 'child-a', parent_id: 'root-a', name: 'A child' },
      ],
      children: [folder('grandchild-a', 'child-a', 'A grandchild')],
      target: file('file-1', 'root-a'),
    });

    expect(browser?.parentId).toBe('root-a');
    expect(browser?.trail.map((entry) => entry.id)).toEqual(['root-a', 'child-a']);
    expect(browser?.children.map((entry) => entry.id)).toEqual(['grandchild-a']);
  });

  it('rejects unresolved directory trails and moving to the current parent', () => {
    const target = file('file-1', 'root-a');

    expect(
      buildMoveDirectoryBrowser({
        currentId: 'missing-folder',
        trail: [],
        children: [],
        target,
      })
    ).toBeNull();
    expect(canMoveToDestination('root-a', target)).toBe(false);
    expect(canMoveToDestination(ROOT_DIRECTORY_ID, target)).toBe(true);
  });
});

function folder(id: string, parent_id: string | null, name: string) {
  return { id, parent_id, name, space_id: 'space-1', type: 'folder' as const };
}

function file(id: string, parent_id: string | null) {
  return { id, parent_id, space_id: 'space-1', type: 'file' as const };
}
