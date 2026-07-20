import type { ReactNode } from 'react';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { vi, it, expect, describe, beforeEach } from 'vitest';

type InitColorSchemeScriptProps = Readonly<{
  attribute?: string;
  defaultMode?: string;
  modeStorageKey?: string;
}>;

const state = vi.hoisted(() => ({
  scriptProps: [] as InitColorSchemeScriptProps[],
}));

vi.mock('src/global.css', () => ({}));

vi.mock('@mui/material/InitColorSchemeScript', async () => {
  const { createElement: create } = await import('react');

  return {
    default: (props: InitColorSchemeScriptProps) => {
      state.scriptProps.push(props);
      return create('script', { 'data-color-scheme-script': 'true' });
    },
  };
});

vi.mock('src/shared/config', () => ({ CONFIG: { assetsDir: '/assets' } }));
vi.mock('src/shared/theme', () => ({ primary: { main: '#000000' } }));
vi.mock('src/shared/theme/theme-config', () => ({
  themeConfig: {
    modeStorageKey: 'theme-mode',
    defaultMode: 'light',
    cssVariables: { colorSchemeSelector: 'data-color-scheme' },
  },
}));

import RootLayout from './layout';

function applicationContent(): ReactNode {
  return createElement('main', { 'data-application': 'true' }, 'application');
}

describe('RootLayout color scheme bootstrap', () => {
  beforeEach(() => {
    state.scriptProps.length = 0;
  });

  it('renders the configured script before application content', () => {
    const markup = renderToStaticMarkup(createElement(RootLayout, null, applicationContent()));

    expect(state.scriptProps).toEqual([
      {
        modeStorageKey: 'theme-mode',
        defaultMode: 'light',
        attribute: 'data-color-scheme',
      },
    ]);
    expect(markup.indexOf('data-color-scheme-script')).toBeLessThan(
      markup.indexOf('data-application')
    );
  });
});
