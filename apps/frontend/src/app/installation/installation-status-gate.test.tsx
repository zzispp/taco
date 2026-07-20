import type { ReactNode } from 'react';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('src/entities/installation', () => ({
  useInstallationStatus: () => ({ kind: 'loading', retry: () => undefined }),
}));

vi.mock('src/shared/i18n', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

vi.mock('src/shared/routes/hooks', () => ({
  useRouter: () => ({ replace: () => undefined }),
}));

vi.mock('src/shared/ui/loading-screen', async () => {
  const { createElement: create } = await import('react');
  return { SplashScreen: () => create('div', { 'data-fallback': 'default-splash' }) };
});

import { InstallationStatusGate } from './installation-status-gate';

function renderGate(loadingFallback?: ReactNode) {
  return renderToStaticMarkup(
    createElement(
      InstallationStatusGate,
      {
        expectedState: 'setup',
        loadingFallback,
        children: createElement('main', null, 'setup-content'),
      }
    )
  );
}

describe('InstallationStatusGate loading fallback', () => {
  it('uses SplashScreen by default for setup routes', () => {
    const markup = renderGate();

    expect(markup).toContain('data-fallback="default-splash"');
    expect(markup).not.toContain('setup-content');
  });

  it('uses the supplied fallback while the installation status is loading', () => {
    const markup = renderGate(createElement('div', { 'data-fallback': 'application-splash' }));

    expect(markup).toContain('data-fallback="application-splash"');
    expect(markup).not.toContain('data-fallback="default-splash"');
  });
});
