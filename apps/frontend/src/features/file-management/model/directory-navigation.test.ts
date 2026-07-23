import { it, expect, describe } from 'vitest';

import {
  fileDirectoryBreadcrumbs,
  fileDirectoryParentTrail,
  isCurrentDirectoryTrailResolved,
} from './directory-navigation';

const DIRECTORY_TRAIL = [
  { id: 'folder-test', parent_id: null, name: '测试' },
  { id: 'folder-test-111', parent_id: 'folder-test', name: '测试 111' },
];

describe('file directory navigation', () => {
  it('builds root, ancestor, and current breadcrumb targets from a resolved directory trail', () => {
    expect(fileDirectoryBreadcrumbs('根目录', DIRECTORY_TRAIL)).toEqual([
      { id: null, name: '根目录', directoryTrail: [] },
      { id: 'folder-test', name: '测试', directoryTrail: ['folder-test'] },
      {
        id: 'folder-test-111',
        name: '测试 111',
        directoryTrail: ['folder-test', 'folder-test-111'],
      },
    ]);
  });

  it('uses the resolved chain rather than route state to find the actual parent', () => {
    expect(fileDirectoryParentTrail(DIRECTORY_TRAIL)).toEqual(['folder-test']);
  });

  it('does not treat an unresolved deep-link directory as navigable', () => {
    expect(isCurrentDirectoryTrailResolved('folder-test-111', [])).toBe(false);
    expect(isCurrentDirectoryTrailResolved('folder-test-111', DIRECTORY_TRAIL)).toBe(true);
  });
});
