import { it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import { SiteDisplayContext } from 'src/shared/config/site-display-context';

import { SiteBrand } from './site-brand';

const SITE_DISPLAY = {
  siteName: 'taco',
  siteSubtitle: 'Backend Control Plane',
  logoUrl: '/logo/logo.svg',
  footerText: 'taco backend control plane.',
};

function renderBrand(showSubtitle?: boolean) {
  return renderToStaticMarkup(
    <SiteDisplayContext value={SITE_DISPLAY}>
      <SiteBrand logoHref="/cn/" showSubtitle={showSubtitle} />
    </SiteDisplayContext>
  );
}

describe('SiteBrand', () => {
  it('shows the configured subtitle when requested by the Dashboard', () => {
    const markup = renderBrand(true);

    expect(markup).toContain('taco');
    expect(markup).toContain('Backend Control Plane');
  });

  it('keeps non-Dashboard brand placements compact by default', () => {
    expect(renderBrand()).not.toContain('Backend Control Plane');
  });
});
