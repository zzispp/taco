import type { EmblaEventType, EmblaCarouselType } from 'embla-carousel';
import type { CarouselOptions } from '../types';

import { useRef, useMemo, useEffect, useCallback } from 'react';

import { setTweenNodes, tweenParallax, setTweenFactor } from './parallax-tween';

const DEFAULT_TWEEN_FACTOR_BASE = 0.24;

export function useParallax(mainApi?: EmblaCarouselType, parallax?: CarouselOptions['parallax']) {
  const tweenFactor = useRef(0);
  const tweenNodes = useRef<HTMLElement[]>([]);
  const baseFactor = typeof parallax === 'number' ? parallax : DEFAULT_TWEEN_FACTOR_BASE;
  const state = useMemo(() => ({ tweenFactor, tweenNodes }), []);

  const handleSetTweenNodes = useCallback(
    (carouselApi: EmblaCarouselType) => setTweenNodes(state, carouselApi),
    [state]
  );
  const handleSetTweenFactor = useCallback(
    (carouselApi: EmblaCarouselType) => setTweenFactor(state, carouselApi, baseFactor),
    [state, baseFactor]
  );
  const handleTweenParallax = useCallback(
    (carouselApi: EmblaCarouselType, eventName?: EmblaEventType) =>
      tweenParallax(state, carouselApi, eventName),
    [state]
  );

  useEffect(() => {
    if (!mainApi || !parallax) return;
    handleSetTweenNodes(mainApi);
    handleSetTweenFactor(mainApi);
    handleTweenParallax(mainApi);
    mainApi
      .on('reInit', handleSetTweenNodes)
      .on('reInit', handleSetTweenFactor)
      .on('reInit', handleTweenParallax)
      .on('scroll', handleTweenParallax)
      .on('slideFocus', handleTweenParallax);
  }, [mainApi, parallax, handleSetTweenNodes, handleSetTweenFactor, handleTweenParallax]);

  return null;
}
