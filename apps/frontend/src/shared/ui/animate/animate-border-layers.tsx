'use client';

import type { RefObject } from 'react';
import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps, CSSObject } from '@mui/material/styles';

import { useRef, useState, useEffect } from 'react';
import {
  m,
  useTransform,
  useMotionValue,
  useAnimationFrame,
  useMotionTemplate,
} from 'framer-motion';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';

import { createClasses } from 'src/shared/theme/create-classes';

// ----------------------------------------------------------------------

const DEFAULT_BORDER_DURATION_SECONDS = 8;
const DEFAULT_BORDER_RX = '30%';
const DEFAULT_BORDER_RY = '30%';
const MILLISECONDS_PER_SECOND = 1_000;
const MOVING_SHAPE_BLUR_RADIUS = 8;
const EMPTY_BORDER_POINT = { x: 0, y: 0 };

const animateBorderLayerClasses = {
  svgWrapper: createClasses('border__animation__svg__wrapper'),
  movingShape: createClasses('border__animation__moving__shape'),
};

export type BorderStyleProps = {
  width?: string;
  size?: number;
  sx?: SxProps<Theme>;
};

export type ComputedBorderStyles = {
  padding: string;
  borderRadius: string;
};

type MovingBorderProps = BoxProps<'span'> & {
  rx?: string;
  ry?: string;
  duration?: number;
  isHidden?: boolean;
  size?: BorderStyleProps['size'];
};

type BorderMotionProps = Pick<MovingBorderProps, 'duration' | 'isHidden' | 'rx' | 'ry'>;

type SecondaryBorderProps = BorderMotionProps & {
  primaryBorder?: BorderStyleProps;
  secondaryBorder?: BorderStyleProps;
  computedStyles: ComputedBorderStyles;
};

export type BorderLayersProps = SecondaryBorderProps & {
  primaryBorderRef: RefObject<HTMLSpanElement | null>;
};

export function BorderLayers({
  rx,
  ry,
  duration,
  isHidden,
  primaryBorder,
  secondaryBorder,
  computedStyles,
  primaryBorderRef,
}: BorderLayersProps) {
  const theme = useTheme();
  const motionProps = { rx, ry, duration, isHidden };

  return (
    <>
      <MovingBorder
        {...motionProps}
        ref={primaryBorderRef}
        size={primaryBorder?.size}
        sx={getPrimaryBorderSx(theme, primaryBorder)}
      />
      <SecondaryBorder
        {...motionProps}
        primaryBorder={primaryBorder}
        secondaryBorder={secondaryBorder}
        computedStyles={computedStyles}
      />
    </>
  );
}

function getPrimaryBorderSx(theme: Theme, border?: BorderStyleProps) {
  return [
    { ...theme.mixins.borderGradient({ padding: border?.width }) },
    ...(Array.isArray(border?.sx) ? border.sx : [border?.sx]),
  ];
}

function SecondaryBorder({
  primaryBorder,
  secondaryBorder,
  computedStyles,
  ...motionProps
}: SecondaryBorderProps) {
  const theme = useTheme();

  if (!secondaryBorder) {
    return null;
  }

  return (
    <MovingBorder
      {...motionProps}
      size={secondaryBorder.size ?? primaryBorder?.size}
      sx={[
        {
          ...theme.mixins.borderGradient({
            padding: secondaryBorder.width ?? computedStyles.padding,
          }),
          borderRadius: computedStyles.borderRadius,
          transform: 'scale(-1, -1)',
        },
        ...(Array.isArray(secondaryBorder.sx) ? secondaryBorder.sx : [secondaryBorder.sx]),
      ]}
    />
  );
}

function MovingBorder({
  sx,
  size,
  isHidden,
  rx = DEFAULT_BORDER_RX,
  ry = DEFAULT_BORDER_RY,
  duration = DEFAULT_BORDER_DURATION_SECONDS,
  ...other
}: MovingBorderProps) {
  const svgRectRef = useRef<SVGRectElement>(null);
  const transform = useMovingBorderTransform({ svgRectRef, duration, isHidden });

  return (
    <Box
      component="span"
      sx={[{ textAlign: 'initial' }, ...(Array.isArray(sx) ? sx : [sx])]}
      {...other}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        preserveAspectRatio="none"
        width="100%"
        height="100%"
        className={animateBorderLayerClasses.svgWrapper}
        style={{ position: 'absolute' }}
      >
        <rect ref={svgRectRef} fill="none" width="100%" height="100%" rx={rx} ry={ry} />
      </svg>

      <Box
        component={m.span}
        style={{ transform }}
        className={animateBorderLayerClasses.movingShape}
        sx={{
          width: size,
          height: size,
          filter: `blur(${MOVING_SHAPE_BLUR_RADIUS}px)`,
          position: 'absolute',
          background: `radial-gradient(currentColor 40%, transparent 80%)`,
        }}
      />
    </Box>
  );
}

type MovingBorderTransformProps = {
  svgRectRef: RefObject<SVGRectElement | null>;
  duration: number;
  isHidden?: boolean;
};

function useMovingBorderTransform({ svgRectRef, duration, isHidden }: MovingBorderTransformProps) {
  const progress = useMotionValue<number>(0);

  useAnimationFrame((time) => {
    const rect = svgRectRef.current;
    const nextProgress =
      rect && !isHidden ? getBorderAnimationProgress(rect, time, duration) : undefined;

    if (nextProgress !== undefined) {
      progress.set(nextProgress);
    }
  });

  const x = useTransform(progress, (value) => getBorderPoint(svgRectRef.current, value).x);
  const y = useTransform(progress, (value) => getBorderPoint(svgRectRef.current, value).y);

  return useMotionTemplate`translateX(${x}px) translateY(${y}px) translateX(-50%) translateY(-50%)`;
}

function getBorderAnimationProgress(rect: SVGRectElement, time: number, duration: number) {
  try {
    const pathLength = rect.getTotalLength();
    const pixelsPerMs = pathLength / (duration * MILLISECONDS_PER_SECOND);

    return (time * pixelsPerMs) % pathLength;
  } catch {
    return undefined;
  }
}

function getBorderPoint(rect: SVGRectElement | null, value: number) {
  if (!rect) {
    return EMPTY_BORDER_POINT;
  }

  try {
    return rect.getPointAtLength(value) ?? EMPTY_BORDER_POINT;
  } catch {
    return EMPTY_BORDER_POINT;
  }
}

export function useComputedElementStyles(
  theme: Theme,
  ref: RefObject<HTMLSpanElement | null>
): ComputedBorderStyles {
  const [computedStyles, setComputedStyles] = useState<CSSObject | null>(null);
  const isRtl = theme.direction === 'rtl';

  useEffect(() => {
    const element = ref.current;

    if (element) {
      const style = getComputedStyle(element);
      setComputedStyles({
        paddingTop: style.paddingBottom,
        paddingBottom: style.paddingTop,
        paddingLeft: isRtl ? style.paddingLeft : style.paddingRight,
        paddingRight: isRtl ? style.paddingRight : style.paddingLeft,
        borderTopLeftRadius: isRtl ? style.borderBottomLeftRadius : style.borderBottomRightRadius,
        borderTopRightRadius: isRtl ? style.borderBottomRightRadius : style.borderBottomLeftRadius,
        borderBottomLeftRadius: isRtl ? style.borderTopLeftRadius : style.borderTopRightRadius,
        borderBottomRightRadius: isRtl ? style.borderTopRightRadius : style.borderTopLeftRadius,
      });
    }
  }, [ref, isRtl]);

  return {
    padding: `${computedStyles?.paddingTop} ${computedStyles?.paddingRight} ${computedStyles?.paddingBottom} ${computedStyles?.paddingLeft}`,
    borderRadius: `${computedStyles?.borderTopLeftRadius} ${computedStyles?.borderTopRightRadius} ${computedStyles?.borderBottomRightRadius} ${computedStyles?.borderBottomLeftRadius}`,
  };
}
