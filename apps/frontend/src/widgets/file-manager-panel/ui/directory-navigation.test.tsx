import type { FileManagerController } from 'src/features/file-management';

import { createElement } from 'react';
import { it, vi, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));
vi.mock('src/shared/ui/iconify', () => ({
  Iconify: () => null,
}));

import { FileManagerDirectoryNavigation } from './directory-navigation';

describe('file manager directory navigation', () => {
  it('keeps parent navigation disabled until a deep-link directory trail resolves', () => {
    const markup = renderToStaticMarkup(
      createElement(FileManagerDirectoryNavigation, {
        controller: controllerWithUnresolvedDirectory(),
      })
    );

    expect(markup).toContain('aria-label="file.actions.upOneLevel"');
    expect(markup).toContain('disabled=""');
  });
});

function controllerWithUnresolvedDirectory() {
  return {
    state: { parentId: 'nested-folder' },
    resources: {
      directoryTrail: [],
      directoryTrailError: undefined,
      directoryTrailLoading: true,
    },
    actions: {
      goToParentFolder: vi.fn(),
      goToDirectory: vi.fn(),
    },
  } as unknown as FileManagerController;
}
