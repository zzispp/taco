'use client';

import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { useMemo } from 'react';
import { useBoolean } from 'minimal-shared/hooks';

import { useTheme } from '@mui/material/styles';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { useSettingsContext } from 'src/shared/ui/settings';

import { useNavbar } from './nav-data';
import { dashboardNavColorVars } from '../ui/css-vars';
import { translateNavData } from '../ui/nav-translation';
import { accountLinksFromNavData } from './account-links';

// ----------------------------------------------------------------------

export function useDashboardLayoutState(navDataOverride?: NavSectionProps['data']) {
  const theme = useTheme();
  const settings = useSettingsContext();
  const { t } = useTranslate('admin');
  const navbar = useNavbar();
  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();
  const sourceNavData = navDataOverride ?? navbar.data;
  const navData = translateNavData(sourceNavData, t);
  const accountLinks = useMemo(
    () => accountLinksFromNavData(navData, t('profile.personalCenter')),
    [navData, t]
  );
  const navLayout = settings.state.navLayout;
  const isNavMini = navLayout === 'mini';
  const isNavHorizontal = navLayout === 'horizontal';
  const isNavVertical = isNavMini || navLayout === 'vertical';
  const navVars = dashboardNavColorVars(theme, settings.state.navColor, navLayout);

  const toggleNav = () => {
    settings.setField('navLayout', navLayout === 'vertical' ? 'mini' : 'vertical');
  };

  return {
    theme,
    navbar,
    navData,
    navVars,
    accountLinks,
    routeGuardNavData: sourceNavData,
    isNavMini,
    isNavHorizontal,
    isNavVertical,
    open,
    onClose,
    onOpen,
    toggleNav,
    t,
  };
}

export type DashboardLayoutState = ReturnType<typeof useDashboardLayoutState>;
