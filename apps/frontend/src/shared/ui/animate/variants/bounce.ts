import type { Variants, Transition } from 'framer-motion';

import { transitionExit, transitionEnter } from './transition';

// ----------------------------------------------------------------------

type Direction =
  | 'in'
  | 'inUp'
  | 'inDown'
  | 'inLeft'
  | 'inRight'
  | 'out'
  | 'outUp'
  | 'outDown'
  | 'outLeft'
  | 'outRight';

type Options = {
  distance?: number;
  transition?: Transition;
};

const DEFAULT_BOUNCE_DISTANCE = 720;

function createBounceVerticalInVariants(
  distance: number,
  transition?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    inUp: {
      initial: {},
      animate: {
        y: [distance, -24, 12, -4, 0],
        scaleY: [4, 0.9, 0.95, 0.985, 1],
        opacity: [0, 1, 1, 1, 1],
        transition: transitionEnter(transition),
      },
    },
    inDown: {
      initial: {},
      animate: {
        y: [-distance, 24, -12, 4, 0],
        scaleY: [4, 0.9, 0.95, 0.985, 1],
        opacity: [0, 1, 1, 1, 1],
        transition: transitionEnter(transition),
      },
    },
  };
}

function createBounceHorizontalInVariants(
  distance: number,
  transition?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    inLeft: {
      initial: {},
      animate: {
        x: [-distance, 24, -12, 4, 0],
        scaleX: [3, 1, 0.98, 0.995, 1],
        opacity: [0, 1, 1, 1, 1],
        transition: transitionEnter(transition),
      },
    },
    inRight: {
      initial: {},
      animate: {
        x: [distance, -24, 12, -4, 0],
        scaleX: [3, 1, 0.98, 0.995, 1],
        opacity: [0, 1, 1, 1, 1],
        transition: transitionEnter(transition),
      },
    },
  };
}

function createBounceInVariants(
  distance: number,
  transition?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    in: {
      initial: {},
      animate: {
        scale: [0.3, 1.1, 0.9, 1.03, 0.97, 1],
        opacity: [0, 1, 1, 1, 1, 1],
        transition: transitionEnter(transition),
      },
    },
    ...createBounceVerticalInVariants(distance, transition),
    ...createBounceHorizontalInVariants(distance, transition),
  };
}

function createBounceOutVariants(
  distance: number,
  transition?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    out: {
      animate: {
        scale: [0.9, 1.1, 0.3],
        opacity: [1, 1, 0],
        transition: transitionExit(transition),
      },
    },
    outUp: {
      animate: {
        y: [-12, 24, -distance],
        scaleY: [0.985, 0.9, 3],
        opacity: [1, 1, 0],
        transition: transitionExit(transition),
      },
    },
    outDown: {
      animate: {
        y: [12, -24, distance],
        scaleY: [0.985, 0.9, 3],
        opacity: [1, 1, 0],
        transition: transitionExit(transition),
      },
    },
    outLeft: {
      animate: {
        x: [0, 24, -distance],
        scaleX: [1, 0.9, 2],
        opacity: [1, 1, 0],
        transition: transitionExit(transition),
      },
    },
    outRight: {
      animate: {
        x: [0, -24, distance],
        scaleX: [1, 0.9, 2],
        opacity: [1, 1, 0],
        transition: transitionExit(transition),
      },
    },
  };
}

export const varBounce = (direction: Direction, options?: Options): Variants => {
  const distance = options?.distance || DEFAULT_BOUNCE_DISTANCE;
  const variants = {
    ...createBounceInVariants(distance, options?.transition),
    ...createBounceOutVariants(distance, options?.transition),
  };
  return variants[direction] as Variants;
};
