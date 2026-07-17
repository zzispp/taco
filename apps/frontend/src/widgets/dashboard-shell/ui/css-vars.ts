import type { Theme, CSSObject } from '@mui/material/styles';
import type { SettingsState } from 'src/shared/ui/settings';

import { varAlpha } from 'minimal-shared/utils';

import { bulletColor } from 'src/shared/ui/nav-section';

// ----------------------------------------------------------------------

export function dashboardLayoutVars(theme: Theme) {
  return {
    '--layout-transition-easing': 'linear',
    '--layout-transition-duration': '120ms',
    '--layout-nav-mini-width': '88px',
    '--layout-nav-vertical-width': '300px',
    '--layout-nav-horizontal-height': '64px',
    '--layout-dashboard-content-pt': theme.spacing(1),
    '--layout-dashboard-content-pb': theme.spacing(8),
    '--layout-dashboard-content-px': theme.spacing(5),
  };
}

// ----------------------------------------------------------------------

export function dashboardNavColorVars(
  theme: Theme,
  navColor: SettingsState['navColor'] = 'integrate',
  navLayout: SettingsState['navLayout'] = 'vertical'
): Record<'layout' | 'section', CSSObject | undefined> {
  switch (navColor) {
    case 'integrate':
      return integrateNavColorVars(theme);
    case 'apparent':
      return apparentNavColorVars(theme, navLayout);
    default:
      throw new Error(`Invalid color: ${navColor}`);
  }
}

function integrateNavColorVars(theme: Theme): Record<'layout' | 'section', CSSObject | undefined> {
  const { palette } = theme.vars;

  return {
    layout: {
      '--layout-nav-bg': palette.background.default,
      '--layout-nav-horizontal-bg': varAlpha(palette.background.defaultChannel, 0.8),
      '--layout-nav-border-color': varAlpha(palette.grey['500Channel'], 0.12),
      '--layout-nav-text-primary-color': palette.text.primary,
      '--layout-nav-text-secondary-color': palette.text.secondary,
      '--layout-nav-text-disabled-color': palette.text.disabled,
      ...theme.applyStyles('dark', {
        '--layout-nav-border-color': varAlpha(palette.grey['500Channel'], 0.08),
        '--layout-nav-horizontal-bg': varAlpha(palette.background.defaultChannel, 0.96),
      }),
    },
    section: undefined,
  };
}

function apparentNavColorVars(
  theme: Theme,
  navLayout: SettingsState['navLayout']
): Record<'layout' | 'section', CSSObject | undefined> {
  return {
    layout: apparentLayoutVars(theme),
    section: apparentSectionVars(theme, navLayout),
  };
}

function apparentLayoutVars(theme: Theme): CSSObject {
  const { palette } = theme.vars;

  return {
    '--layout-nav-bg': palette.background.neutral,
    '--layout-nav-horizontal-bg': varAlpha(palette.background.neutralChannel, 0.96),
    '--layout-nav-border-color': varAlpha(palette.grey['500Channel'], 0.24),
    '--layout-nav-text-primary-color': palette.text.primary,
    '--layout-nav-text-secondary-color': palette.text.secondary,
    '--layout-nav-text-disabled-color': palette.text.disabled,
    ...theme.applyStyles('dark', {
      '--layout-nav-bg': palette.grey[800],
      '--layout-nav-horizontal-bg': varAlpha(palette.grey['800Channel'], 0.8),
      '--layout-nav-border-color': 'transparent',
      '--layout-nav-text-primary-color': palette.common.white,
      '--layout-nav-text-secondary-color': palette.grey[500],
      '--layout-nav-text-disabled-color': palette.grey[600],
    }),
  };
}

function apparentSectionVars(theme: Theme, navLayout: SettingsState['navLayout']): CSSObject {
  const { palette } = theme.vars;

  return {
    '--nav-item-caption-color': palette.text.secondary,
    '--nav-subheader-color': palette.text.secondary,
    '--nav-subheader-hover-color': palette.text.primary,
    '--nav-item-color': palette.text.secondary,
    '--nav-item-root-active-color': palette.primary.dark,
    '--nav-item-root-open-color': palette.text.primary,
    '--nav-bullet-light-color': bulletColor.light,
    ...(navLayout === 'vertical' && {
      '--nav-item-sub-active-color': palette.text.primary,
      '--nav-item-sub-open-color': palette.text.primary,
    }),
    ...theme.applyStyles('dark', {
      '--nav-item-caption-color': palette.grey[600],
      '--nav-subheader-color': palette.grey[600],
      '--nav-subheader-hover-color': palette.common.white,
      '--nav-item-color': palette.grey[500],
      '--nav-item-root-active-color': palette.primary.light,
      '--nav-item-root-open-color': palette.common.white,
      '--nav-bullet-light-color': bulletColor.dark,
      ...(navLayout === 'vertical' && {
        '--nav-item-sub-active-color': palette.common.white,
        '--nav-item-sub-open-color': palette.common.white,
      }),
    }),
  };
}
