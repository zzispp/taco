import { it, expect, describe } from 'vitest';

import { paths } from 'src/shared/routes/paths';

import {
  systemMenuItemTranslationKey,
  systemMenuSectionTranslationKey,
} from './system-menu-translation';

describe('system menu translation', () => {
  it('translates the file-management section seeded by its menu id', () => {
    expect(systemMenuSectionTranslationKey('5')).toBe('nav.fileManagement');
  });

  it('translates each file-management route', () => {
    expect(systemMenuItemTranslationKey(undefined, paths.dashboard.file)).toBe('nav.fileOverview');
    expect(systemMenuItemTranslationKey(undefined, paths.dashboard.fileManager)).toBe(
      'nav.fileManager'
    );
    expect(systemMenuItemTranslationKey(undefined, paths.dashboard.fileSpaces)).toBe(
      'nav.fileSpaces'
    );
  });
});
