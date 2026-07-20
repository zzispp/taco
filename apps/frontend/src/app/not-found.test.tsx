import { createElement } from 'react';
import { it, vi, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('src/app/providers/error-page-providers', () => ({
  ErrorPageProviders: ({ children }: { children: React.ReactNode }) => children,
}));

vi.mock('src/pages-layer/error-404', () => ({
  Error404Page: ({ homeHref }: { homeHref: string }) =>
    createElement('main', { 'data-home-href': homeHref, 'data-template-error-page': 'true' }),
}));

import NotFound from './not-found';

describe('root not-found page', () => {
  it('uses the project error page with an explicit default locale home path', () => {
    const markup = renderToStaticMarkup(createElement(NotFound));

    expect(markup).toContain('data-template-error-page="true"');
    expect(markup).toContain('data-home-href="/cn/"');
  });
});
