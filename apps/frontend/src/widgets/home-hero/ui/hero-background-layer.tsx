import type { Theme } from '@mui/material/styles';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';

import { CONFIG } from 'src/shared/config';

export function HeroBackgroundLayer() {
  return (
    <Box
      component={m.div}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      sx={[
        (theme) => ({
          ...backgroundGradient(theme),
          ...backgroundPositionSx,
          ...darkBackgroundGradient(theme),
        }),
      ]}
    />
  );
}

const backgroundPositionSx = {
  top: 0,
  left: 0,
  width: 1,
  height: 1,
  zIndex: -1,
  position: 'absolute',
} as const;

function backgroundGradient(theme: Theme) {
  return theme.mixins.bgGradient({
    images: [
      `linear-gradient(180deg, ${theme.vars.palette.background.default} 12%, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.92)} 50%, ${theme.vars.palette.background.default} 88%)`,
      `url(${CONFIG.assetsDir}/assets/background/background-3.webp)`,
    ],
  });
}

function darkBackgroundGradient(theme: Theme) {
  return theme.applyStyles('dark', {
    ...theme.mixins.bgGradient({
      images: [
        `url(${CONFIG.assetsDir}/assets/images/home/hero-blur.webp)`,
        `linear-gradient(180deg, ${theme.vars.palette.background.default} 12%, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.96)} 50%, ${theme.vars.palette.background.default} 88%)`,
        `url(${CONFIG.assetsDir}/assets/background/background-3.webp)`,
      ],
    }),
  });
}
