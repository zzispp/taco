import { m } from 'framer-motion';

import Box from '@mui/material/Box';

import { Lines, Circles, PlusIcon } from './hero-svg';

const HERO_WIDTH = 1440;
const HERO_HEIGHT = 1080;
const LINE_STROKE_COUNT = 12;

export function HeroBackgroundSvg() {
  return (
    <Box
      component={m.svg}
      xmlns="http://www.w3.org/2000/svg"
      width={HERO_WIDTH}
      height={HERO_HEIGHT}
      fill="none"
      viewBox="0 0 1440 1080"
      initial="hidden"
      animate="visible"
      sx={[{ width: 1, height: 1 }]}
    >
      <HeroMask />
      <g mask="url(#mask_id)">
        <Circles />
        <PlusIcon />
        <Lines strokeCount={LINE_STROKE_COUNT} />
      </g>
    </Box>
  );
}

function HeroMask() {
  return (
    <defs>
      <radialGradient
        id="mask_gradient_id"
        cx="0"
        cy="0"
        r="1"
        gradientTransform="matrix(720 0 0 420 720 560)"
        gradientUnits="userSpaceOnUse"
      >
        <stop offset="0%" stopColor="#FFFFFF" stopOpacity={1} />
        <stop offset="100%" stopColor="#FFFFFF" stopOpacity={0.08} />
      </radialGradient>
      <mask id="mask_id">
        <ellipse cx="50%" cy="50%" rx="50%" ry="36%" fill="url(#mask_gradient_id)" />
      </mask>
    </defs>
  );
}
