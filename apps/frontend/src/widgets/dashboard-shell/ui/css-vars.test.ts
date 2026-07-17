import type { Theme } from '@mui/material/styles';

import { it, expect, describe } from 'vitest';
import { varAlpha } from 'minimal-shared/utils';

import { dashboardNavColorVars } from './css-vars';

const PALETTE = {
  background: {
    default: '#ffffff',
    defaultChannel: '255 255 255',
    neutral: '#edf1f5',
    neutralChannel: '237 241 245',
  },
  common: { white: '#ffffff' },
  grey: {
    500: '#6f7a86',
    600: '#4f5a66',
    800: '#28323d',
    900: '#1d252d',
    '500Channel': '111 122 134',
    '800Channel': '40 50 61',
    '900Channel': '29 37 45',
  },
  primary: { dark: '#0059b3', light: '#6db6ff' },
  text: { disabled: '#8d99a6', primary: '#1d252d', secondary: '#4f5a66' },
};

describe('apparent Dashboard navigation colors', () => {
  it('uses a neutral high-contrast navigation in light mode', () => {
    const vars = dashboardNavColorVars(theme('light'), 'apparent', 'vertical');

    expect(vars.layout).toMatchObject({
      '--layout-nav-bg': PALETTE.background.neutral,
      '--layout-nav-border-color': varAlpha(PALETTE.grey['500Channel'], 0.24),
      '--layout-nav-text-primary-color': PALETTE.text.primary,
      '--layout-nav-text-secondary-color': PALETTE.text.secondary,
    });
    expect(vars.section).toMatchObject({
      '--nav-item-color': PALETTE.text.secondary,
      '--nav-item-root-active-color': PALETTE.primary.dark,
      '--nav-item-root-open-color': PALETTE.text.primary,
      '--nav-item-sub-active-color': PALETTE.text.primary,
    });
  });

  it('keeps the dark high-contrast navigation in dark mode', () => {
    const vars = dashboardNavColorVars(theme('dark'), 'apparent', 'vertical');

    expect(vars.layout).toMatchObject({
      '--layout-nav-bg': PALETTE.grey[800],
      '--layout-nav-text-primary-color': PALETTE.common.white,
    });
    expect(vars.section).toMatchObject({
      '--nav-item-color': PALETTE.grey[500],
      '--nav-item-root-active-color': PALETTE.primary.light,
      '--nav-item-root-open-color': PALETTE.common.white,
      '--nav-item-sub-active-color': PALETTE.common.white,
    });
  });
});

function theme(mode: 'light' | 'dark') {
  return {
    vars: { palette: PALETTE },
    applyStyles: (scheme: 'dark', styles: object) => (scheme === mode ? styles : {}),
  } as unknown as Theme;
}
