'use client';

import type { EmblaPluginType } from 'embla-carousel';
import type { CarouselOptions, UseCarouselReturn } from '../types';

import useEmblaCarousel from 'embla-carousel-react';

import { useTheme } from '@mui/material/styles';

import { useThumbs } from './use-thumbs';
import { useCarouselDots } from './use-carousel-dots';
import { useParallax } from './use-carousel-parallax';
import { useCarouselArrows } from './use-carousel-arrows';
import { useCarouselControls } from './use-carousel-controls';
import { useCarouselProgress } from './use-carousel-progress';
import { useCarouselAutoplay } from './use-carousel-autoplay';
import { useCarouselAutoScroll } from './use-carousel-auto-scroll';

// ----------------------------------------------------------------------

export function useCarousel(
  options?: CarouselOptions,
  plugins?: EmblaPluginType[]
): UseCarouselReturn {
  const theme = useTheme();

  const [mainRef, mainApi] = useEmblaCarousel({ ...options, direction: theme.direction }, plugins);

  const pluginNames = plugins?.map((plugin) => plugin.name);

  const dots = useCarouselDots(mainApi);
  const arrows = useCarouselArrows(mainApi);
  const progress = useCarouselProgress(mainApi);
  const autoplay = useCarouselAutoplay(mainApi);
  const autoScroll = useCarouselAutoScroll(mainApi);
  const thumbs = useThumbs(mainApi, options?.thumbs);

  useParallax(mainApi, options?.parallax);

  const controls = useCarouselControls({ pluginNames, arrows, autoplay, autoScroll });

  const mergedOptions = { ...options, ...mainApi?.internalEngine().options };

  return {
    options: mergedOptions,
    pluginNames,
    mainRef,
    mainApi,
    arrows: {
      disablePrev: arrows.disablePrev,
      disableNext: arrows.disableNext,
      onClickPrev: controls.onClickPrev,
      onClickNext: controls.onClickNext,
    },
    dots,
    thumbs,
    progress,
    autoplay,
    autoScroll,
  };
}
