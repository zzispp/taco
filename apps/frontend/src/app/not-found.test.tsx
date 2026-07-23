import { it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import GlobalNotFound from './global-not-found';

describe('global not-found page', () => {
  it('uses the project error page with an explicit default locale home path', async () => {
    const page = await GlobalNotFound();
    const markup = renderToStaticMarkup(page);

    expect(markup.match(/<html/g)).toHaveLength(1);
    expect(markup.match(/<body/g)).toHaveLength(1);
    expect(markup).toContain('<html lang="zh-CN"');
    expect(markup).toContain('data-template-error-page="true"');
    expect(markup).toContain('data-home-href="/cn/"');
    expect(markup).toContain('抱歉，页面未找到！');
  });
});
