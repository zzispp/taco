'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { SimpleCompactContentProps } from './content';
import type {
  MainSectionProps,
  HeaderSectionProps,
  LayoutSectionProps,
} from 'src/shared/ui/layout';

import { merge } from 'es-toolkit';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';

import { SiteBrand } from 'src/shared/ui/logo';
import { paths } from 'src/shared/routes/paths';
import { RouterLink } from 'src/shared/routes/components';
import { SettingsButton } from 'src/shared/ui/shell/settings-button';
import { MainSection, LayoutSection, HeaderSection } from 'src/shared/ui/layout';

import { SimpleCompactContent } from './content';

// ----------------------------------------------------------------------

type LayoutBaseProps = Pick<LayoutSectionProps, 'sx' | 'children' | 'cssVars'>;

export type SimpleLayoutProps = LayoutBaseProps & {
  homeHref?: string;
  layoutQuery?: Breakpoint;
  slotProps?: {
    header?: HeaderSectionProps;
    main?: MainSectionProps;
    content?: SimpleCompactContentProps & { compact?: boolean };
  };
};

export function SimpleLayout({
  sx,
  cssVars,
  children,
  slotProps,
  homeHref = paths.home,
  layoutQuery = 'md',
}: SimpleLayoutProps) {
  return (
    <LayoutSection
      headerSection={
        <SimpleHeader homeHref={homeHref} layoutQuery={layoutQuery} slotProps={slotProps} />
      }
      footerSection={null}
      cssVars={{ '--layout-simple-content-compact-width': '448px', ...cssVars }}
      sx={sx}
    >
      <SimpleMain layoutQuery={layoutQuery} slotProps={slotProps}>
        {children}
      </SimpleMain>
    </LayoutSection>
  );
}

type SimpleLayoutPartProps = {
  layoutQuery: Breakpoint;
  slotProps?: SimpleLayoutProps['slotProps'];
};

function SimpleHeader({
  homeHref,
  layoutQuery,
  slotProps,
}: SimpleLayoutPartProps & { homeHref: string }) {
  const headerSlotProps: HeaderSectionProps['slotProps'] = { container: { maxWidth: false } };
  const headerSlots: HeaderSectionProps['slots'] = {
    topArea: (
      <Alert severity="info" sx={{ display: 'none', borderRadius: 0 }}>
        This is an info Alert.
      </Alert>
    ),
    leftArea: <SiteBrand logoHref={homeHref} />,
    rightArea: (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: { xs: 1, sm: 1.5 } }}>
        <Link
          href={homeHref}
          component={RouterLink}
          color="inherit"
          sx={{ typography: 'subtitle2' }}
        >
          Back to home
        </Link>
        <SettingsButton />
      </Box>
    ),
  };

  return (
    <HeaderSection
      layoutQuery={layoutQuery}
      {...slotProps?.header}
      slots={{ ...headerSlots, ...slotProps?.header?.slots }}
      slotProps={merge(headerSlotProps, slotProps?.header?.slotProps ?? {})}
      sx={slotProps?.header?.sx}
    />
  );
}

function SimpleMain({
  layoutQuery,
  slotProps,
  children,
}: SimpleLayoutPartProps & { children: React.ReactNode }) {
  const { compact, ...contentProps } = slotProps?.content ?? {};

  return (
    <MainSection {...slotProps?.main}>
      {compact ? (
        <SimpleCompactContent layoutQuery={layoutQuery} {...contentProps}>
          {children}
        </SimpleCompactContent>
      ) : (
        children
      )}
    </MainSection>
  );
}
