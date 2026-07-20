import type { ReactNode } from 'react';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { vi, it, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({
  gateAllowsInstalledProviders: false,
  initializedProviders: [] as string[],
}));

type ChildrenProps = Readonly<{
  children?: ReactNode;
  loadingFallback?: ReactNode;
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
vi.mock('src/shared/ui/loading-screen', async () => {
  const { createElement: create } = await import('react');
  return { SplashScreen: () => create('div', { 'data-provider': 'splash' }) };
});

vi.mock('src/app/installation/installation-status-gate', async () => {
  const { createElement: create } = await import('react');
  return {
    InstallationStatusGate: ({ children, loadingFallback }: ChildrenProps) =>
      create(
        'div',
        { 'data-provider': 'setup-gate' },
        state.gateAllowsInstalledProviders ? children : loadingFallback
      ),
  };
});

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
    createElement(ApplicationProviders, null, createElement('main', null, 'installed-content'))
  );
}

describe('ApplicationProviders', () => {
  beforeEach(() => {
    state.gateAllowsInstalledProviders = false;
    state.initializedProviders.length = 0;
  });

  it('uses a local loading theme without initializing installed-only providers', () => {
    const markup = renderProviders();

    expect(markup).toMatch(
      /data-provider="setup-gate"><div data-provider="cache"><div data-provider="theme"><div data-provider="splash"/
    );
    expect(markup.match(/data-provider="theme"/g)).toHaveLength(1);
    expect(markup.match(/data-provider="cache"/g)).toHaveLength(1);
    expect(state.initializedProviders).toEqual([]);
    expect(markup).not.toContain('installed-content');
  });

  it('initializes the single runtime theme after the installation gate allows rendering', () => {
    state.gateAllowsInstalledProviders = true;

    const markup = renderProviders();

    expect(state.initializedProviders).toEqual(['auth', 'app-settings']);
    expect(markup).toMatch(
      /data-provider="setup-gate"><div data-provider="auth"><div data-provider="app-settings"><div data-provider="localization"><div data-provider="cache"><div data-provider="theme">/
    );
    expect(markup.match(/data-provider="theme"/g)).toHaveLength(1);
    expect(markup).not.toContain('data-provider="splash"');
    expect(markup).toContain('installed-content');
  });
});
