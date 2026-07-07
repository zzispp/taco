import type { Theme } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

// ----------------------------------------------------------------------

export function megaMenuVars(theme: Theme, variant: 'vertical' | 'horizontal' | 'mobile') {
  const {
    vars: { palette },
  } = theme;

  return {
    ...megaMenuLayoutVars(theme, variant),
    '--nav-icon-size': '22px',
    '--nav-icon-margin': variantValue(variant, {
      mobile: '0 16px 0 0',
      vertical: '0 16px 0 0',
      horizontal: '0 8px 0 0',
    }),
    '--nav-item-hover-bg': palette.action.hover,
    '--nav-item-active-color': palette.primary.main,
    '--nav-item-active-bg': variantValue(variant, {
      mobile: varAlpha(palette.primary.mainChannel, 0.08),
      vertical: varAlpha(palette.primary.mainChannel, 0.08),
      horizontal: 'transparent',
    }),
    '--nav-item-active-hover-bg': variantValue(variant, {
      mobile: varAlpha(palette.primary.mainChannel, 0.16),
      vertical: varAlpha(palette.primary.mainChannel, 0.16),
      horizontal: varAlpha(palette.primary.mainChannel, 0.08),
    }),
  };
}

type MegaMenuVariant = 'vertical' | 'horizontal' | 'mobile';
type VariantValues = Partial<Record<MegaMenuVariant, string | number>>;

function megaMenuLayoutVars(theme: Theme, variant: MegaMenuVariant) {
  return {
    '--nav-width': variantValue(variant, {
      mobile: '280px',
      vertical: '260px',
      horizontal: 'unset',
    }),
    '--nav-item-gap': variantValue(variant, {
      mobile: theme.spacing(0.5),
      vertical: theme.spacing(0.5),
      horizontal: theme.spacing(2.5),
    }),
    '--nav-item-radius': variantValue(variant, {
      mobile: '0',
      vertical: '0',
      horizontal: `${theme.shape.borderRadius}px`,
    }),
    '--nav-item-height': variantValue(variant, {
      mobile: '40px',
      vertical: '40px',
      horizontal: '32px',
    }),
    '--nav-item-padding': variantValue(variant, {
      mobile: theme.spacing(1, 1.5, 1, 2.5),
      vertical: theme.spacing(1, 1.5, 1, 2.5),
      horizontal: theme.spacing(0.5, 1),
    }),
  };
}

function variantValue(variant: MegaMenuVariant, values: VariantValues) {
  return values[variant];
}
