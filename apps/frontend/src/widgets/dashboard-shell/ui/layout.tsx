'use client';

import type { Breakpoint } from '@mui/material/styles';
import type { NavSectionProps } from 'src/shared/ui/nav-section';
import type {
  MainSectionProps,
  HeaderSectionProps,
  LayoutSectionProps,
} from 'src/shared/ui/layout';

import { DashboardLayoutFrame } from './layout-sections';
import { useDashboardLayoutState } from '../model/layout-state';

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
  const state = useDashboardLayoutState(slotProps?.nav?.data);

  return (
    <DashboardLayoutFrame
      sx={sx}
      cssVars={cssVars}
      slotProps={slotProps}
      layoutQuery={layoutQuery}
      state={state}
    >
      {children}
    </DashboardLayoutFrame>
  );
}
