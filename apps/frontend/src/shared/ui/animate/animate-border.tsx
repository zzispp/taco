'use client';

import type { ReactNode, RefObject } from 'react';
import type { BoxProps } from '@mui/material/Box';
import type { Theme } from '@mui/material/styles';

import { mergeClasses } from 'minimal-shared/utils';
import { useRef, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';

import { createClasses } from 'src/shared/theme/create-classes';

import {
  BorderLayers,
  type BorderStyleProps,
  useComputedElementStyles,
  type ComputedBorderStyles,
} from './animate-border-layers';

// ----------------------------------------------------------------------

const MIN_BORDER_DIMENSION = 40;

const animateBorderClasses = {
  root: createClasses('border__animation__root'),
};

type AnimateBorderProps = BoxProps & {
  duration?: number;
  slotProps?: {
    primaryBorder?: BorderStyleProps;
    secondaryBorder?: BorderStyleProps;
    outlineColor?: string | ((theme: Theme) => string);
    svgSettings?: {
      rx?: string;
      ry?: string;
    };
  };
};

type BorderSurfaceProps = AnimateBorderProps & {
  theme: Theme;
  rootRef: RefObject<HTMLDivElement | null>;
  primaryBorderRef: RefObject<HTMLSpanElement | null>;
  isHidden: boolean;
  secondaryBorderStyles: ComputedBorderStyles;
};

function resolveOutlineColor(
  outlineColor: NonNullable<AnimateBorderProps['slotProps']>['outlineColor'],
  theme: Theme
) {
  return typeof outlineColor === 'function' ? outlineColor(theme) : outlineColor;
}

function useBorderVisibility(rootRef: RefObject<HTMLDivElement | null>) {
  const [isHidden, setIsHidden] = useState(false);

  useEffect(() => {
    const updateVisibility = () => {
      const root = rootRef.current;

      if (root) {
        setIsHidden(getComputedStyle(root).display === 'none');
      }
    };

    updateVisibility();
    window.addEventListener('resize', updateVisibility);

    return () => window.removeEventListener('resize', updateVisibility);
  }, [rootRef]);

  return isHidden;
}

export function AnimateBorder(props: AnimateBorderProps) {
  const theme = useTheme();
  const rootRef = useRef<HTMLDivElement>(null);
  const primaryBorderRef = useRef<HTMLSpanElement>(null);
  const isHidden = useBorderVisibility(rootRef);
  const secondaryBorderStyles = useComputedElementStyles(theme, primaryBorderRef);

  return (
    <BorderSurface
      {...props}
      theme={theme}
      rootRef={rootRef}
      primaryBorderRef={primaryBorderRef}
      isHidden={isHidden}
      secondaryBorderStyles={secondaryBorderStyles}
    />
  );
}

function BorderSurface({
  sx,
  theme,
  children,
  duration,
  slotProps,
  className,
  rootRef,
  isHidden,
  primaryBorderRef,
  secondaryBorderStyles,
  ...other
}: BorderSurfaceProps) {
  const primaryBorder = slotProps?.primaryBorder;
  const motionProps = {
    duration,
    isHidden,
    rx: slotProps?.svgSettings?.rx,
    ry: slotProps?.svgSettings?.ry,
  };

  return (
    <Box
      dir="ltr"
      ref={rootRef}
      className={mergeClasses([animateBorderClasses.root, className])}
      sx={getRootSx({ theme, sx, children, outlineColor: slotProps?.outlineColor, primaryBorder })}
      {...other}
    >
      <BorderLayers
        {...motionProps}
        primaryBorder={primaryBorder}
        secondaryBorder={slotProps?.secondaryBorder}
        computedStyles={secondaryBorderStyles}
        primaryBorderRef={primaryBorderRef}
      />
      {children}
    </Box>
  );
}

function getRootSx({
  theme,
  sx,
  children,
  outlineColor,
  primaryBorder,
}: {
  theme: Theme;
  sx: AnimateBorderProps['sx'];
  children: ReactNode;
  outlineColor: NonNullable<AnimateBorderProps['slotProps']>['outlineColor'];
  primaryBorder?: BorderStyleProps;
}) {
  return [
    {
      minWidth: MIN_BORDER_DIMENSION,
      minHeight: MIN_BORDER_DIMENSION,
      overflow: 'hidden',
      position: 'relative',
      width: 'fit-content',
      '&::before': theme.mixins.borderGradient({
        color: resolveOutlineColor(outlineColor, theme),
        padding: primaryBorder?.width,
      }),
      ...(children && { minWidth: 'unset', minHeight: 'unset' }),
    },
    ...(Array.isArray(sx) ? sx : [sx]),
  ];
}
