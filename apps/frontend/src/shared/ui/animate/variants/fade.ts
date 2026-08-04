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
  transitionIn?: Transition;
  transitionOut?: Transition;
};

const DEFAULT_FADE_DISTANCE = 120;

function createFadeInVariants(
  distance: number,
  transitionIn?: Transition,
  transitionOut?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    in: {
      initial: { opacity: 0 },
      animate: { opacity: 1, transition: transitionEnter() },
      exit: { opacity: 0, transition: transitionExit() },
    },
    inUp: {
      initial: { y: distance, opacity: 0 },
      animate: { y: 0, opacity: 1, transition: transitionEnter(transitionIn) },
      exit: { y: distance, opacity: 0, transition: transitionExit(transitionOut) },
    },
    inDown: {
      initial: { y: -distance, opacity: 0 },
      animate: { y: 0, opacity: 1, transition: transitionEnter(transitionIn) },
      exit: { y: -distance, opacity: 0, transition: transitionExit(transitionOut) },
    },
    inLeft: {
      initial: { x: -distance, opacity: 0 },
      animate: { x: 0, opacity: 1, transition: transitionEnter(transitionIn) },
      exit: { x: -distance, opacity: 0, transition: transitionExit(transitionOut) },
    },
    inRight: {
      initial: { x: distance, opacity: 0 },
      animate: { x: 0, opacity: 1, transition: transitionEnter(transitionIn) },
      exit: { x: distance, opacity: 0, transition: transitionExit(transitionOut) },
    },
  };
}

function createFadeOutVariants(
  distance: number,
  transitionIn?: Transition,
  transitionOut?: Transition
): Partial<Record<Direction, Variants>> {
  return {
    out: {
      initial: { opacity: 1 },
      animate: { opacity: 0, transition: transitionEnter(transitionIn) },
      exit: { opacity: 1, transition: transitionExit(transitionOut) },
    },
    outUp: {
      initial: { y: 0, opacity: 1 },
      animate: { y: -distance, opacity: 0, transition: transitionEnter(transitionIn) },
      exit: { y: 0, opacity: 1, transition: transitionExit(transitionOut) },
    },
    outDown: {
      initial: { y: 0, opacity: 1 },
      animate: { y: distance, opacity: 0, transition: transitionEnter(transitionIn) },
      exit: { y: 0, opacity: 1, transition: transitionExit(transitionOut) },
    },
    outLeft: {
      initial: { x: 0, opacity: 1 },
      animate: { x: -distance, opacity: 0, transition: transitionEnter(transitionIn) },
      exit: { x: 0, opacity: 1, transition: transitionExit(transitionOut) },
    },
    outRight: {
      initial: { x: 0, opacity: 1 },
      animate: { x: distance, opacity: 0, transition: transitionEnter(transitionIn) },
      exit: { x: 0, opacity: 1, transition: transitionExit(transitionOut) },
    },
  };
}

export const varFade = (direction: Direction, options?: Options): Variants => {
  const distance = options?.distance || DEFAULT_FADE_DISTANCE;
  const variants = {
    ...createFadeInVariants(distance, options?.transitionIn, options?.transitionOut),
    ...createFadeOutVariants(distance, options?.transitionIn, options?.transitionOut),
  };
  return variants[direction] as Variants;
};
