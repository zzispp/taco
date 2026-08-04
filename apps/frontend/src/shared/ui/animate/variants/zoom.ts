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

const DEFAULT_ZOOM_DISTANCE = 720;

type ZoomAxis = 'translateX' | 'translateY';

type ZoomVariantOptions = {
  axis: ZoomAxis;
  value: number;
  transitionIn?: Transition;
  transitionOut?: Transition;
};

function createZoomOffset(axis: ZoomAxis, value: number) {
  return axis === 'translateX' ? { translateX: value } : { translateY: value };
}

function createZoomInVariant(options: ZoomVariantOptions): Variants {
  const offset = createZoomOffset(options.axis, options.value);
  return {
    initial: { scale: 0, opacity: 0, ...offset },
    animate: {
      scale: 1,
      opacity: 1,
      ...createZoomOffset(options.axis, 0),
      transition: transitionEnter(options.transitionIn),
    },
    exit: { scale: 0, opacity: 0, ...offset, transition: transitionExit(options.transitionOut) },
  };
}

function createZoomOutVariant(options: Omit<ZoomVariantOptions, 'transitionOut'>): Variants {
  return {
    initial: { scale: 1, opacity: 1 },
    animate: {
      scale: 0,
      opacity: 0,
      ...createZoomOffset(options.axis, options.value),
      transition: transitionEnter(options.transitionIn),
    },
  };
}

function createZoomVariants(
  distance: number,
  transitionIn?: Transition,
  transitionOut?: Transition
): Record<Direction, Variants> {
  return {
    in: {
      initial: { scale: 0, opacity: 0 },
      animate: { scale: 1, opacity: 1, transition: transitionEnter(transitionIn) },
      exit: { scale: 0, opacity: 0, transition: transitionExit(transitionOut) },
    },
    inUp: createZoomInVariant({ axis: 'translateY', value: distance, transitionIn, transitionOut }),
    inDown: createZoomInVariant({
      axis: 'translateY',
      value: -distance,
      transitionIn,
      transitionOut,
    }),
    inLeft: createZoomInVariant({
      axis: 'translateX',
      value: -distance,
      transitionIn,
      transitionOut,
    }),
    inRight: createZoomInVariant({
      axis: 'translateX',
      value: distance,
      transitionIn,
      transitionOut,
    }),
    out: {
      initial: { scale: 1, opacity: 1 },
      animate: { scale: 0, opacity: 0, transition: transitionEnter(transitionIn) },
    },
    outUp: createZoomOutVariant({ axis: 'translateY', value: -distance, transitionIn }),
    outDown: createZoomOutVariant({ axis: 'translateY', value: distance, transitionIn }),
    outLeft: createZoomOutVariant({ axis: 'translateX', value: -distance, transitionIn }),
    outRight: createZoomOutVariant({ axis: 'translateX', value: distance, transitionIn }),
  };
}

export const varZoom = (direction: Direction, options?: Options): Variants => {
  const distance = options?.distance || DEFAULT_ZOOM_DISTANCE;
  return createZoomVariants(distance, options?.transitionIn, options?.transitionOut)[direction];
};
