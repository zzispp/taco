import type { BoxProps } from '@mui/material/Box';
import type { Theme } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import useMediaQuery from '@mui/material/useMediaQuery';

import { MotionContainer } from 'src/shared/ui/animate';

import { Dots, Texts } from './hero-svg';
import { HeroBackgroundSvg } from './hero-background-svg';
import { HeroBackgroundLayer } from './hero-background-layer';

type HeroBackgroundProps = BoxProps & {
  text: string;
};

export function HeroBackground({ sx, text, ...other }: HeroBackgroundProps) {
  const mdUp = useMediaQuery((theme) => theme.breakpoints.up('md'));

  return (
    <MotionContainer>
      <Box sx={[(theme) => heroRootSx(theme), ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
        <Dots />
        {mdUp && <Texts text={text} />}
        <HeroBackgroundSvg />
        <HeroBackgroundLayer />
      </Box>
    </MotionContainer>
  );
}

function heroRootSx(theme: Theme) {
  return {
    '--stroke-dasharray': 3,
    '--stroke-spacing': '80px',
    '--hero-line-stroke-width': 1,
    '--hero-line-stroke-color': varAlpha(theme.vars.palette.grey['500Channel'], 0.32),
    '--hero-text-stroke-width': 1,
    '--hero-text-stroke-color': varAlpha(theme.vars.palette.grey['500Channel'], 0.24),
    '--hero-circle-stroke-width': 1,
    '--hero-circle-stroke-color': varAlpha(theme.vars.palette.grey['500Channel'], 0.48),
    '--hero-plus-stroke-color': theme.vars.palette.text.disabled,
    top: 0,
    left: 0,
    width: 1,
    height: 1,
    position: 'absolute',
    ...theme.applyStyles('dark', {
      '--hero-line-stroke-color': varAlpha(theme.vars.palette.grey['600Channel'], 0.16),
      '--hero-text-stroke-color': varAlpha(theme.vars.palette.grey['600Channel'], 0.12),
      '--hero-circle-stroke-color': varAlpha(theme.vars.palette.grey['600Channel'], 0.24),
    }),
  };
}
