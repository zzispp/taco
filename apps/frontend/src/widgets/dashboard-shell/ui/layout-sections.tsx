import type { Theme, Breakpoint } from '@mui/material/styles';
import type { DashboardLayoutProps } from './layout';
import type { HeaderSectionProps } from 'src/shared/ui/layout';
import type { DashboardLayoutState } from '../model/layout-state';

import { merge } from 'es-toolkit';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import { iconButtonClasses } from '@mui/material/IconButton';

import { allLangs } from 'src/shared/i18n';
import { SiteBrand } from 'src/shared/ui/logo';
import { MenuButton } from 'src/shared/ui/shell/menu-button';
import { SettingsButton } from 'src/shared/ui/shell/settings-button';
import { LanguagePopover } from 'src/shared/ui/shell/language-popover';
import { MainSection, layoutClasses, HeaderSection, LayoutSection } from 'src/shared/ui/layout';

import { NAV_ICONS } from 'src/entities/menu';

import { NotificationsDrawer } from 'src/features/notice-reading';

import { Searchbar } from './searchbar';
import { NavMobile } from './nav-mobile';
import { VerticalDivider } from './content';
import { NavVertical } from './nav-vertical';
import { NavHorizontal } from './nav-horizontal';
import { AccountDrawer } from './account-drawer';
import { dashboardLayoutVars } from './css-vars';
import { DashboardRouteGuard } from './route-guard';

// ----------------------------------------------------------------------

type DashboardLayoutFrameProps = Pick<
  DashboardLayoutProps,
  'sx' | 'cssVars' | 'children' | 'slotProps'
> & {
  layoutQuery: Breakpoint;
  state: DashboardLayoutState;
};

export function DashboardLayoutFrame({
  sx,
  cssVars,
  children,
  slotProps,
  layoutQuery,
  state,
}: DashboardLayoutFrameProps) {
  return (
    <LayoutSection
      headerSection={
        <DashboardHeader layoutQuery={layoutQuery} header={slotProps?.header} state={state} />
      }
      sidebarSection={
        state.isNavHorizontal ? null : <DashboardSidebar layoutQuery={layoutQuery} state={state} />
      }
      footerSection={null}
      cssVars={{ ...dashboardLayoutVars(state.theme), ...state.navVars.layout, ...cssVars }}
      sx={getDashboardLayoutSx({ theme: state.theme, layoutQuery, isNavMini: state.isNavMini, sx })}
    >
      <MainSection {...slotProps?.main}>
        <DashboardRouteGuard
          data={state.routeGuardNavData}
          error={state.navbar.error}
          isLoading={!slotProps?.nav?.data && state.navbar.isLoading}
        >
          {children}
        </DashboardRouteGuard>
      </MainSection>
    </LayoutSection>
  );
}

type DashboardHeaderProps = {
  layoutQuery: Breakpoint;
  header?: HeaderSectionProps;
  state: DashboardLayoutState;
};

function DashboardHeader({ layoutQuery, header, state }: DashboardHeaderProps) {
  const headerSlotProps = getHeaderSlotProps(state, layoutQuery);
  const headerSlots: HeaderSectionProps['slots'] = {
    topArea: <DashboardHeaderNotice />,
    bottomArea: <DashboardHeaderNavigation state={state} layoutQuery={layoutQuery} />,
    leftArea: <DashboardHeaderLeft state={state} layoutQuery={layoutQuery} />,
    rightArea: <DashboardHeaderActions state={state} />,
  };

  return (
    <HeaderSection
      layoutQuery={layoutQuery}
      disableElevation={state.isNavVertical}
      {...header}
      slots={{ ...headerSlots, ...header?.slots }}
      slotProps={merge(headerSlotProps, header?.slotProps ?? {})}
      sx={header?.sx}
    />
  );
}

function getHeaderSlotProps(state: DashboardLayoutState, layoutQuery: Breakpoint) {
  return {
    container: {
      maxWidth: false,
      sx: {
        ...(state.isNavVertical && { px: { [layoutQuery]: 5 } }),
        ...(state.isNavHorizontal && {
          bgcolor: 'var(--layout-nav-bg)',
          height: { [layoutQuery]: 'var(--layout-nav-horizontal-height)' },
          [`& .${iconButtonClasses.root}`]: { color: 'var(--layout-nav-text-secondary-color)' },
        }),
      },
    },
  } satisfies HeaderSectionProps['slotProps'];
}

function DashboardHeaderNotice() {
  return (
    <Alert severity="info" sx={{ display: 'none', borderRadius: 0 }}>
      This is an info Alert.
    </Alert>
  );
}

function DashboardHeaderNavigation({ state, layoutQuery }: Omit<DashboardHeaderProps, 'header'>) {
  if (!state.isNavHorizontal) {
    return null;
  }

  return (
    <NavHorizontal
      data={state.navData}
      render={{ navIcon: NAV_ICONS }}
      layoutQuery={layoutQuery}
      cssVars={state.navVars.section}
    />
  );
}

function DashboardHeaderLeft({ state, layoutQuery }: Omit<DashboardHeaderProps, 'header'>) {
  return (
    <>
      <MenuButton
        onClick={state.onOpen}
        sx={{ mr: 1, ml: -1, [state.theme.breakpoints.up(layoutQuery)]: { display: 'none' } }}
      />
      <NavMobile
        data={state.navData}
        open={state.open}
        onClose={state.onClose}
        render={{ navIcon: NAV_ICONS }}
        cssVars={state.navVars.section}
      />
      {state.isNavHorizontal && (
        <SiteBrand
          showSubtitle
          sx={{ display: 'none', [state.theme.breakpoints.up(layoutQuery)]: { display: 'flex' } }}
        />
      )}
      {state.isNavHorizontal && (
        <VerticalDivider sx={{ [state.theme.breakpoints.up(layoutQuery)]: { display: 'flex' } }} />
      )}
    </>
  );
}

function DashboardHeaderActions({ state }: Pick<DashboardHeaderProps, 'state'>) {
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 0, sm: 0.75 } }}>
      <Searchbar data={state.navData} />
      <LanguagePopover data={allLangs} />
      <NotificationsDrawer />
      <SettingsButton />
      <AccountDrawer data={state.accountLinks} navTitle={state.t('profile.authorizedNavigation')} />
    </Box>
  );
}

function DashboardSidebar({ layoutQuery, state }: Omit<DashboardHeaderProps, 'header'>) {
  return (
    <NavVertical
      data={state.navData}
      render={{ navIcon: NAV_ICONS }}
      isNavMini={state.isNavMini}
      layoutQuery={layoutQuery}
      cssVars={state.navVars.section}
      onToggleNav={state.toggleNav}
    />
  );
}

function getDashboardLayoutSx({
  theme,
  layoutQuery,
  isNavMini,
  sx,
}: {
  theme: Theme;
  layoutQuery: Breakpoint;
  isNavMini: boolean;
  sx: DashboardLayoutProps['sx'];
}) {
  return [
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
  ];
}
