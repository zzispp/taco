'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { AuthSplitSectionProps } from './section';
import type { AuthSplitContentProps } from './content';
import type {
  MainSectionProps,
  HeaderSectionProps,
  LayoutSectionProps,
} from 'src/shared/ui/layout';

import { merge } from 'es-toolkit';

import Box from '@mui/material/Box';

import { allLangs } from 'src/shared/i18n';
import { SiteBrand } from 'src/shared/ui/logo';
import { SettingsButton } from 'src/shared/ui/shell/settings-button';
import { LanguagePopover } from 'src/shared/ui/shell/language-popover';
import { MainSection, LayoutSection, HeaderSection } from 'src/shared/ui/layout';

import { AuthSplitSection } from './section';
import { AuthSplitContent } from './content';

// ----------------------------------------------------------------------

type LayoutBaseProps = Pick<LayoutSectionProps, 'sx' | 'children' | 'cssVars'>;

export type AuthSplitLayoutProps = LayoutBaseProps & {
  layoutQuery?: Breakpoint;
  slotProps?: {
    header?: HeaderSectionProps;
    main?: MainSectionProps;
    section?: AuthSplitSectionProps;
    content?: AuthSplitContentProps;
  };
};

export function AuthSplitLayout({
  sx,
  cssVars,
  children,
  slotProps,
  layoutQuery = 'md',
}: AuthSplitLayoutProps) {
  return (
    <LayoutSection
      headerSection={<AuthHeader layoutQuery={layoutQuery} slotProps={slotProps} />}
      footerSection={null}
      cssVars={{ '--layout-auth-content-width': '420px', ...cssVars }}
      sx={sx}
    >
      <AuthMain layoutQuery={layoutQuery} slotProps={slotProps}>
        {children}
      </AuthMain>
    </LayoutSection>
  );
}

type AuthLayoutPartProps = {
  layoutQuery: Breakpoint;
  slotProps?: AuthSplitLayoutProps['slotProps'];
};

function AuthHeader({ layoutQuery, slotProps }: AuthLayoutPartProps) {
  const headerSlotProps: HeaderSectionProps['slotProps'] = { container: { maxWidth: false } };
  const headerSlots: HeaderSectionProps['slots'] = {
    leftArea: <SiteBrand />,
    rightArea: (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 1, sm: 1.5 } }}>
        <LanguagePopover data={allLangs} />
        <SettingsButton />
      </Box>
    ),
  };

  return (
    <HeaderSection
      disableElevation
      layoutQuery={layoutQuery}
      {...slotProps?.header}
      slots={{ ...headerSlots, ...slotProps?.header?.slots }}
      slotProps={merge(headerSlotProps, slotProps?.header?.slotProps ?? {})}
      sx={[
        { position: { [layoutQuery]: 'fixed' } },
        ...(Array.isArray(slotProps?.header?.sx) ? slotProps.header.sx : [slotProps?.header?.sx]),
      ]}
    />
  );
}

function AuthMain({
  layoutQuery,
  slotProps,
  children,
}: AuthLayoutPartProps & { children: React.ReactNode }) {
  return (
    <MainSection
      {...slotProps?.main}
      sx={[
        (theme) => ({ [theme.breakpoints.up(layoutQuery)]: { flexDirection: 'row' } }),
        ...(Array.isArray(slotProps?.main?.sx) ? slotProps.main.sx : [slotProps?.main?.sx]),
      ]}
    >
      <AuthSplitSection layoutQuery={layoutQuery} {...slotProps?.section} />
      <AuthSplitContent layoutQuery={layoutQuery} {...slotProps?.content}>
        {children}
      </AuthSplitContent>
    </MainSection>
  );
}
