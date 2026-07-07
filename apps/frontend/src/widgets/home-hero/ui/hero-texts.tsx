import type { MotionProps } from 'framer-motion';
import type { BoxProps } from '@mui/material/Box';
import type { Theme } from '@mui/material/styles';

import { m } from 'framer-motion';

import Box from '@mui/material/Box';

import { varFade } from 'src/shared/ui/animate';

const TEXT_SCROLL_DURATION = 64;
const TEXT_REPEAT_COUNT = 2;

type TextsProps = BoxProps &
  MotionProps & {
    text: string;
  };

export function Texts({ sx, text, ...other }: TextsProps) {
  return (
    <Box
      component={m.div}
      variants={varFade('in')}
      sx={[containerSx, ...(Array.isArray(sx) ? sx : [sx])]}
      {...other}
    >
      <Box component="svg" sx={[textSvgSx]}>
        <m.text
          x="0"
          y="12px"
          dominantBaseline="hanging"
          animate={{ x: ['0%', '-50%'] }}
          transition={{ duration: TEXT_SCROLL_DURATION, ease: 'linear', repeat: Infinity }}
        >
          {Array(TEXT_REPEAT_COUNT).fill(text).join(' ')}
        </m.text>
      </Box>
    </Box>
  );
}

const containerSx = {
  left: 0,
  width: 1,
  bottom: 0,
  height: 200,
  position: 'absolute',
} as const;

const textSvgSx = (theme: Theme) => ({
  width: 1,
  height: 1,
  '& text': {
    fill: 'none',
    fontSize: 200,
    fontWeight: 800,
    strokeDasharray: 4,
    textTransform: 'uppercase',
    stroke: 'var(--hero-text-stroke-color)',
    strokeWidth: 'var(--hero-text-stroke-width)',
    fontFamily: theme.typography.fontSecondaryFamily,
  },
});
