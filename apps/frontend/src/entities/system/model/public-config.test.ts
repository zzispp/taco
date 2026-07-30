import { it, expect, describe } from 'vitest';

import { PUBLIC_CONFIG_KEYS, siteDisplayFromPublicConfigs } from './public-config';

const SITE_DISPLAY_CONFIG = {
  site_name: 'taco',
  site_subtitle: 'Backend Control Plane',
  logo_url: '/logo/logo.svg',
  footer_text: 'taco backend control plane.',
};

describe('site display public config', () => {
  it('parses the configured site subtitle', () => {
    const result = siteDisplayFromPublicConfigs({
      [PUBLIC_CONFIG_KEYS.siteDisplayConfig]: JSON.stringify(SITE_DISPLAY_CONFIG),
    });

    expect(result).toEqual(SITE_DISPLAY_CONFIG);
  });

  it('rejects a site display config without a subtitle', () => {
    const configWithoutSubtitle = {
      site_name: SITE_DISPLAY_CONFIG.site_name,
      logo_url: SITE_DISPLAY_CONFIG.logo_url,
      footer_text: SITE_DISPLAY_CONFIG.footer_text,
    };

    expect(() =>
      siteDisplayFromPublicConfigs({
        [PUBLIC_CONFIG_KEYS.siteDisplayConfig]: JSON.stringify(configWithoutSubtitle),
      })
    ).toThrow('Invalid public system config');
  });
});
