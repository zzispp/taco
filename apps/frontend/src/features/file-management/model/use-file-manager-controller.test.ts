import { it, expect, describe } from 'vitest';

import { resolveFileManagerSpaceId } from './use-file-manager-controller';

describe('file manager space resolution', () => {
  it('uses the current user virtual space when list permission cannot load an overview', () => {
    expect(resolveFileManagerSpaceId(undefined, undefined, 'user-1')).toBe('user-1');
  });

  it('keeps an explicitly selected managed space ahead of the overview space', () => {
    expect(resolveFileManagerSpaceId('selected-space', 'overview-space', 'user-1')).toBe(
      'selected-space'
    );
  });
});
