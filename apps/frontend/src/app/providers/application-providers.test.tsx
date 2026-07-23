import type { ReactNode } from 'react';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { vi, it, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({
  initializedProviders: [] as string[],
}));

type ChildrenProps = Readonly<{
  children?: ReactNode;
}>;

vi.mock('@mui/material-nextjs/v15-appRouter', async () => {
  const { createElement: create } = await import('react');
  return {
    AppRouterCacheProvider: ({ children }: ChildrenProps) =>
      create('div', { 'data-provider': 'cache' }, children),
  };
});

vi.mock('src/shared/i18n', async () => {
  const { createElement: create } = await import('react');
  return {
    LocalizationProvider: ({ children }: ChildrenProps) =>
      create('div', { 'data-provider': 'localization' }, children),
  };
});

vi.mock('src/shared/i18n/i18n-provider', async () => {
  const { createElement: create } = await import('react');
  return {
    I18nProvider: ({ children }: ChildrenProps) =>
      create('div', { 'data-provider': 'i18n' }, children),
  };
});

vi.mock('src/shared/theme', async () => {
  const { createElement: create } = await import('react');
  return {
    themeConfig: {
      modeStorageKey: 'theme-mode',
      defaultMode: 'light',
      cssVariables: { colorSchemeSelector: 'data-mui-color-scheme' },
    },
    ThemeProvider: ({ children }: ChildrenProps) =>
      create('div', { 'data-provider': 'theme' }, children),
  };
});

vi.mock('src/shared/ui/settings', async () => {
  const { createElement: create } = await import('react');
  return {
    defaultSettings: {},
    SettingsDrawer: () => null,
    SettingsProvider: ({ children }: ChildrenProps) =>
      create('div', { 'data-provider': 'settings' }, children),
  };
});

vi.mock('src/shared/ui/animate/motion-lazy', async () => {
  const { createElement: create } = await import('react');
  return { MotionLazy: ({ children }: ChildrenProps) => create('div', null, children) };
});

vi.mock('src/shared/ui/progress-bar', () => ({ ProgressBar: () => null }));
vi.mock('src/shared/ui/snackbar', () => ({ Snackbar: () => null }));

vi.mock('./auth-provider', async () => {
  const { createElement: create } = await import('react');
  return {
    AuthProvider: ({ children }: ChildrenProps) => {
      state.initializedProviders.push('auth');
      return create('div', { 'data-provider': 'auth' }, children);
    },
  };
});

vi.mock('./settings-provider', async () => {
  const { createElement: create } = await import('react');
  return {
    AppSettingsProvider: ({ children }: ChildrenProps) => {
      state.initializedProviders.push('app-settings');
      return create('div', { 'data-provider': 'app-settings' }, children);
    },
  };
});

import { ApplicationProviders } from './application-providers';

function renderProviders() {
  return renderToStaticMarkup(
    createElement(ApplicationProviders, null, createElement('main', null, 'application-content'))
  );
}

describe('ApplicationProviders', () => {
  beforeEach(() => {
    state.initializedProviders.length = 0;
  });

  it('initializes the runtime providers immediately', () => {
    const markup = renderProviders();

    expect(state.initializedProviders).toEqual(['auth', 'app-settings']);
    expect(markup).toMatch(
      /data-provider="settings"><div data-provider="auth"><div data-provider="app-settings"><div data-provider="localization"><div data-provider="cache"><div data-provider="theme">/
    );
    expect(markup.match(/data-provider="theme"/g)).toHaveLength(1);
    expect(markup).toContain('application-content');
  });
});
