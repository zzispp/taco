'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { NavSectionProps } from 'src/shared/ui/nav-section';
import type {
  MainSectionProps,
  HeaderSectionProps,
  LayoutSectionProps,
} from 'src/shared/ui/layout';

import { useMemo } from 'react';
import { merge } from 'es-toolkit';
import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import { useTheme } from '@mui/material/styles';
import { iconButtonClasses } from '@mui/material/IconButton';

import { Logo } from 'src/shared/ui/logo';
import { allLangs } from 'src/shared/i18n';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { useSettingsContext } from 'src/shared/ui/settings';
import { MenuButton } from 'src/shared/ui/shell/menu-button';
import { SettingsButton } from 'src/shared/ui/shell/settings-button';
import { LanguagePopover } from 'src/shared/ui/shell/language-popover';
import { MainSection, layoutClasses, HeaderSection, LayoutSection } from 'src/shared/ui/layout';

import { NAV_ICONS } from 'src/entities/menu';

import { Searchbar } from 'src/widgets/dashboard-shell/ui/searchbar';
import { useNavbar } from 'src/widgets/dashboard-shell/model/nav-data';
import { AccountDrawer } from 'src/widgets/dashboard-shell/ui/account-drawer';
import { accountLinksFromNavData } from 'src/widgets/dashboard-shell/model/account-links';

import { NavMobile } from './nav-mobile';
import { VerticalDivider } from './content';
import { NavVertical } from './nav-vertical';
import { NavHorizontal } from './nav-horizontal';
import { DashboardRouteGuard } from './route-guard';
import { translateNavData } from './nav-translation';
import { dashboardLayoutVars, dashboardNavColorVars } from './css-vars';

// ----------------------------------------------------------------------

type LayoutBaseProps = Pick<LayoutSectionProps, 'sx' | 'children' | 'cssVars'>;

export type DashboardLayoutProps = LayoutBaseProps & {
  layoutQuery?: Breakpoint;
  slotProps?: {
    header?: HeaderSectionProps;
    nav?: {
      data?: NavSectionProps['data'];
    };
    main?: MainSectionProps;
  };
};

export function DashboardLayout({
  sx,
  cssVars,
  children,
  slotProps,
  layoutQuery = 'lg',
}: DashboardLayoutProps) {
  const theme = useTheme();
  const settings = useSettingsContext();
  const { t } = useTranslate('admin');
  const navbar = useNavbar();

  const navVars = dashboardNavColorVars(theme, settings.state.navColor, settings.state.navLayout);
  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();

  const sourceNavData = slotProps?.nav?.data ?? navbar.data;
  const routeGuardNavData = slotProps?.nav?.data ?? navbar.data;
  const navData = translateNavData(sourceNavData, t);
  const accountLinks = useMemo(
    () => accountLinksFromNavData(navData, t('profile.personalCenter')),
    [navData, t]
  );

  const isNavMini = settings.state.navLayout === 'mini';
  const isNavHorizontal = settings.state.navLayout === 'horizontal';
  const isNavVertical = isNavMini || settings.state.navLayout === 'vertical';

  const renderHeader = () => {
    const headerSlotProps: HeaderSectionProps['slotProps'] = {
      container: {
        maxWidth: false,
        sx: {
          ...(isNavVertical && { px: { [layoutQuery]: 5 } }),
          ...(isNavHorizontal && {
            bgcolor: 'var(--layout-nav-bg)',
            height: { [layoutQuery]: 'var(--layout-nav-horizontal-height)' },
            [`& .${iconButtonClasses.root}`]: { color: 'var(--layout-nav-text-secondary-color)' },
          }),
        },
      },
    };

    const headerSlots: HeaderSectionProps['slots'] = {
      topArea: (
        <Alert severity="info" sx={{ display: 'none', borderRadius: 0 }}>
          This is an info Alert.
        </Alert>
      ),
      bottomArea: isNavHorizontal ? (
        <NavHorizontal
          data={navData}
          render={{ navIcon: NAV_ICONS }}
          layoutQuery={layoutQuery}
          cssVars={navVars.section}
        />
      ) : null,
      leftArea: (
        <>
          <MenuButton
            onClick={onOpen}
            sx={{ mr: 1, ml: -1, [theme.breakpoints.up(layoutQuery)]: { display: 'none' } }}
          />
          <NavMobile
            data={navData}
            open={open}
            onClose={onClose}
            render={{ navIcon: NAV_ICONS }}
            cssVars={navVars.section}
          />

          {isNavHorizontal && (
            <Logo
              sx={{
                display: 'none',
                [theme.breakpoints.up(layoutQuery)]: { display: 'inline-flex' },
              }}
            />
          )}

          {isNavHorizontal && (
            <VerticalDivider sx={{ [theme.breakpoints.up(layoutQuery)]: { display: 'flex' } }} />
          )}
        </>
      ),
      rightArea: (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 0, sm: 0.75 } }}>
          <Searchbar data={navData} />
          <LanguagePopover data={allLangs} />
          <SettingsButton />
          <AccountDrawer data={accountLinks} navTitle={t('profile.authorizedNavigation')} />
        </Box>
      ),
    };

    return (
      <HeaderSection
        layoutQuery={layoutQuery}
        disableElevation={isNavVertical}
        {...slotProps?.header}
        slots={{ ...headerSlots, ...slotProps?.header?.slots }}
        slotProps={merge(headerSlotProps, slotProps?.header?.slotProps ?? {})}
        sx={slotProps?.header?.sx}
      />
    );
  };

  const renderSidebar = () => (
    <NavVertical
      data={navData}
      render={{ navIcon: NAV_ICONS }}
      isNavMini={isNavMini}
      layoutQuery={layoutQuery}
      cssVars={navVars.section}
      onToggleNav={() =>
        settings.setField(
          'navLayout',
          settings.state.navLayout === 'vertical' ? 'mini' : 'vertical'
        )
      }
    />
  );

  return (
    <LayoutSection
      headerSection={renderHeader()}
      sidebarSection={isNavHorizontal ? null : renderSidebar()}
      footerSection={null}
      cssVars={{ ...dashboardLayoutVars(theme), ...navVars.layout, ...cssVars }}
      sx={[
        {
          [`& .${layoutClasses.sidebarContainer}`]: {
            [theme.breakpoints.up(layoutQuery)]: {
              pl: isNavMini ? 'var(--layout-nav-mini-width)' : 'var(--layout-nav-vertical-width)',
              transition: theme.transitions.create(['padding-left'], {
                easing: 'var(--layout-transition-easing)',
                duration: 'var(--layout-transition-duration)',
              }),
            },
          },
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      <MainSection {...slotProps?.main}>
        <DashboardRouteGuard
          data={routeGuardNavData}
          error={navbar.error}
          isLoading={!slotProps?.nav?.data && navbar.isLoading}
        >
          {children}
        </DashboardRouteGuard>
      </MainSection>
    </LayoutSection>
  );
}
