import { useMemo } from 'react';

export type CarouselControlSources = {
  pluginNames?: string[];
  arrows: {
    onClickPrev: () => void;
    onClickNext: () => void;
  };
  autoplay: {
    onClickPlay: (callback: () => void) => void;
  };
  autoScroll: {
    onClickPlay: (callback: () => void) => void;
  };
};

export function useCarouselControls(sources: CarouselControlSources) {
  return useMemo(() => {
    if (sources.pluginNames?.includes('autoplay')) return autoplayControls(sources);
    if (sources.pluginNames?.includes('autoScroll')) return autoScrollControls(sources);
    return sources.arrows;
  }, [sources]);
}

function autoplayControls({ arrows, autoplay }: CarouselControlSources) {
  return {
    onClickPrev: () => autoplay.onClickPlay(arrows.onClickPrev),
    onClickNext: () => autoplay.onClickPlay(arrows.onClickNext),
  };
}

function autoScrollControls({ arrows, autoScroll }: CarouselControlSources) {
  return {
    onClickPrev: () => autoScroll.onClickPlay(arrows.onClickPrev),
    onClickNext: () => autoScroll.onClickPlay(arrows.onClickNext),
  };
}
