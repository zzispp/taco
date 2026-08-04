'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { ContainerProps } from '@mui/material/Container';

import { mergeClasses } from 'minimal-shared/utils';

import Container from '@mui/material/Container';

import { layoutClasses } from 'src/shared/ui/layout';
import { useSettingsContext } from 'src/shared/ui/settings';

export type DashboardContentProps = ContainerProps & {
  layoutQuery?: Breakpoint;
  disablePadding?: boolean;
};

export function DashboardContent({
  sx,
  children,
  className,
  disablePadding,
  maxWidth = 'lg',
  layoutQuery = 'lg',
  ...other
}: DashboardContentProps) {
  const settings = useSettingsContext();
  const isNavHorizontal = settings.state.navLayout === 'horizontal';

  return (
    <Container
      className={mergeClasses([layoutClasses.content, className])}
      maxWidth={settings.state.compactLayout ? maxWidth : false}
      sx={[
        (theme) => ({
          display: 'flex',
          flex: '1 1 auto',
          flexDirection: 'column',
          pt: 'var(--layout-dashboard-content-pt)',
          pb: 'var(--layout-dashboard-content-pb)',
          [theme.breakpoints.up(layoutQuery)]: {
            px: 'var(--layout-dashboard-content-px)',
            ...(isNavHorizontal && { '--layout-dashboard-content-pt': '40px' }),
          },
          ...(disablePadding && {
            p: {
              xs: 0,
              sm: 0,
              md: 0,
              lg: 0,
              xl: 0,
            },
          }),
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      {children}
    </Container>
  );
}
