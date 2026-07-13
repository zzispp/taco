import { it, expect, describe } from 'vitest';

import { canPreviewCron } from './permissions';

describe('scheduler editor permissions', () => {
  it.each([
    [{ canImport: true, canEdit: false }, true],
    [{ canImport: false, canEdit: true }, true],
    [{ canImport: true, canEdit: true }, true],
    [{ canImport: false, canEdit: false }, false],
  ] as const)('allows cron preview for import or edit permission', (permissions, expected) => {
    expect(canPreviewCron(permissions)).toBe(expected);
  });
});
