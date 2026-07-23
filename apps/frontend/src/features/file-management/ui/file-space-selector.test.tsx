import type { FileSpaceSelectorState } from '../model/use-file-space-selector';

import { createElement } from 'react';
import { it, vi, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

vi.mock('src/shared/ui/iconify', () => ({
  Iconify: () => null,
}));

import { FileSpaceSelector } from './file-space-selector';

describe('file space selector', () => {
  it('keeps its cursor controls in the same toolbar row as the space field', () => {
    const markup = renderToStaticMarkup(
      createElement(FileSpaceSelector, {
        selector: selectorState(),
        currentUserId: 'user-1',
        label: 'file.fields.space',
        onChange: vi.fn(),
      })
    );

    expect(rootStackRule(markup)).toContain('flex-direction:row');
    expect(rootStackRule(markup)).toContain('align-items:center');
  });
});

function rootStackRule(markup: string): string {
  const className = markup.match(
    /^<style[^>]*>.*?<\/style><div class="MuiStack-root ([^"]+)"/
  )?.[1];
  const emotionClass = className?.split(' ').find((item) => item.startsWith('css-'));
  const rule = emotionClass
    ? markup.match(new RegExp(`\\.${emotionClass}\\{([^}]*)\\}`))?.[1]
    : undefined;
  if (!rule) throw new Error('file space selector root style rule is unavailable');
  return rule;
}

function selectorState(): FileSpaceSelectorState {
  return {
    selectedSpace: null,
    selectedSpaceId: undefined,
    setSearch: vi.fn(),
    rememberSpace: vi.fn(),
    table: { onPreviousCursor: vi.fn(), onNextCursor: vi.fn() },
    spaces: {
      items: [],
      isValidating: false,
      hasPrevious: false,
      hasNext: false,
      previousCursor: undefined,
      nextCursor: undefined,
    },
  } as unknown as FileSpaceSelectorState;
}
