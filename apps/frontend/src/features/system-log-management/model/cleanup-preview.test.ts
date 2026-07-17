import { it, expect, describe } from 'vitest';

import { createSystemLogCleanupPreview, acceptsSystemLogCleanupPreview } from './cleanup-preview';

describe('system log cleanup preview', () => {
  it('freezes the displayed query so confirmation cannot expand its scope', () => {
    const query = { keyword: 'narrow', begin_time: '2026-07-15T00:00:00.000Z' };
    const preview = createSystemLogCleanupPreview(query, 3);

    query.keyword = 'broad';

    expect(preview).toEqual({
      query: { keyword: 'narrow', begin_time: '2026-07-15T00:00:00.000Z' },
      count: 3,
    });
    expect(() => {
      (preview.query as Record<string, string>).keyword = 'different';
    }).toThrow(TypeError);
  });

  it('ignores an in-flight preview response after filters invalidate it', () => {
    expect(acceptsSystemLogCleanupPreview(3, 3)).toBe(true);
    expect(acceptsSystemLogCleanupPreview(4, 3)).toBe(false);
  });
});
