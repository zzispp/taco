import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps } from '@mui/material/styles';
import type { Transition, MotionProps } from 'framer-motion';
import type { PaletteColorKey } from 'src/shared/theme/core';

import { m } from 'framer-motion';

import Box from '@mui/material/Box';

const DOT_ANIMATION_DURATION = 6;

type DotProps = Pick<MotionProps, 'animate' | 'transition'> & {
  sx?: SxProps<Theme>;
  color?: PaletteColorKey;
};

export function Dots() {
  return (
    <>
      <Dot color="error" animate={{ x: [0, 24] }} sx={dotSx(14, 'translate(calc(50% - 457px), calc(50% - 259px))')} />
      <Dot color="warning" animate={{ y: [0, 24] }} sx={dotSx(12, 'translate(calc(50% - 356px), calc(50% + 37px))')} />
      <Dot color="info" animate={{ x: [0, 24] }} sx={dotSx(12, 'translate(calc(50% + 332px), calc(50% + 135px))')} />
      <Dot color="secondary" animate={{ x: [0, 24] }} sx={dotSx(12, 'translate(calc(50% + 430px), calc(50% - 160px))')} />
      <Dot color="success" animate={{ y: [0, 24] }} sx={dotSx(12, 'translate(calc(50% + 136px), calc(50% + 332px))')} />
    </>
  );
}

function Dot({ color = 'primary', animate, transition, sx, ...other }: DotProps & BoxProps) {
  return (
    <Box component={m.div} variants={dotVariants} sx={[dotContainerSx, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <Box
        component={m.div}
        animate={animate}
        transition={transition ?? defaultDotTransition()}
        sx={[(theme) => dotInnerSx(theme, color), ...(Array.isArray(sx) ? sx : [sx])]}
      />
    </Box>
  );
}

const dotVariants = {
  initial: { opacity: 0 },
  animate: { opacity: 1, transition: { duration: 0.64, ease: [0.43, 0.13, 0.23, 0.96] } },
};

const dotContainerSx = {
  width: 12,
  height: 12,
  top: '50%',
  left: '50%',
  position: 'absolute',
} as const;

function dotSx(size: number, transform: string) {
  return { width: size, height: size, transform };
}

function defaultDotTransition(): Transition {
  return {
    duration: DOT_ANIMATION_DURATION,
    ease: 'linear',
    repeat: Infinity,
    repeatType: 'reverse' as const,
  };
}

function dotInnerSx(theme: Theme, color: PaletteColorKey) {
  return {
    width: 1,
    height: 1,
    borderRadius: '50%',
    boxShadow: `0px -2px 4px 0px ${theme.vars.palette[color].main} inset`,
    background: `linear-gradient(135deg, ${theme.vars.palette[color].lighter}, ${theme.vars.palette[color].light})`,
    ...theme.applyStyles('dark', {
      boxShadow: `0px -2px 4px 0px ${theme.vars.palette[color].dark} inset`,
    }),
  };
}
