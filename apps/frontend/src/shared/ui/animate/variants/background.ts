import type { Variants, Transition, TargetAndTransition } from 'framer-motion';

// ----------------------------------------------------------------------

type Direction = 'top' | 'bottom' | 'left' | 'right';

const ANIMATION_DURATION_SECONDS = 5;

type PanSettings = {
  angle: number;
  startPosition: string;
  endPosition: string;
  backgroundSize: string;
};

const panSettings: Record<Direction, PanSettings> = {
  top: {
    angle: 0,
    startPosition: 'center 99%',
    endPosition: 'center 1%',
    backgroundSize: '100% 600%',
  },
  right: {
    angle: 270,
    startPosition: '1% center',
    endPosition: '99% center',
    backgroundSize: '600% 100%',
  },
  bottom: {
    angle: 0,
    startPosition: 'center 1%',
    endPosition: 'center 99%',
    backgroundSize: '100% 600%',
  },
  left: {
    angle: 270,
    startPosition: '99% center',
    endPosition: '1% center',
    backgroundSize: '600% 100%',
  },
};

export const varBgColor = (colors: string[], options?: TargetAndTransition): Variants => ({
  animate: {
    background: colors,
    ...options,
    transition: {
      duration: ANIMATION_DURATION_SECONDS,
      ease: 'linear',
      repeat: Infinity,
      repeatType: 'reverse',
      ...options?.transition,
    },
  },
});

// ----------------------------------------------------------------------

export const varBgKenburns = (direction: Direction, options?: TargetAndTransition): Variants => {
  const transition: Transition = {
    duration: ANIMATION_DURATION_SECONDS,
    ease: 'easeOut',
    ...options?.transition,
  };

  const variants: Record<Direction, Variants> = {
    top: {
      animate: {
        scale: [1, 1.25],
        y: [0, -15],
        transformOrigin: ['50% 16%', '50% top'],
        ...options,
        transition,
      },
    },
    bottom: {
      animate: {
        scale: [1, 1.25],
        y: [0, 15],
        transformOrigin: ['50% 84%', '50% bottom'],
        ...options,
        transition,
      },
    },
    left: {
      animate: {
        scale: [1, 1.25],
        x: [0, 20],
        y: [0, 15],
        transformOrigin: ['16% 50%', '0% left'],
        ...options,
        transition,
      },
    },
    right: {
      animate: {
        scale: [1, 1.25],
        x: [0, -20],
        y: [0, -15],
        transformOrigin: ['84% 50%', '0% right'],
        ...options,
        transition,
      },
    },
  };

  return variants[direction];
};

// ----------------------------------------------------------------------

export const varBgPan = (
  direction: Direction,
  colors: string[],
  options?: TargetAndTransition
): Variants => {
  const settings = panSettings[direction];
  const gradient = `linear-gradient(${settings.angle}deg, ${colors.join(', ')})`;
  const transition: Transition = {
    duration: ANIMATION_DURATION_SECONDS,
    ease: 'linear',
    repeat: Infinity,
    repeatType: 'reverse',
    ...options?.transition,
  };

  return {
    animate: {
      backgroundImage: [gradient, gradient],
      backgroundPosition: [settings.startPosition, settings.endPosition],
      backgroundSize: [settings.backgroundSize, settings.backgroundSize],
      ...options,
      transition,
    },
  };
};
