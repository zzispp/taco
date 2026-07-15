import type { EmblaEventType, EmblaCarouselType } from 'embla-carousel';

import { carouselClasses } from '../classes';

export type ParallaxState = {
  tweenFactor: React.MutableRefObject<number>;
  tweenNodes: React.MutableRefObject<HTMLElement[]>;
};

export function setTweenNodes(state: ParallaxState, carouselApi: EmblaCarouselType): void {
  state.tweenNodes.current = carouselApi
    .slideNodes()
    .map(
      (slideNode) => slideNode.querySelector(`.${carouselClasses.slide.parallax}`) as HTMLElement
    );
}

export function setTweenFactor(
  state: ParallaxState,
  carouselApi: EmblaCarouselType,
  baseFactor: number
) {
  state.tweenFactor.current = baseFactor * carouselApi.scrollSnapList().length;
}

export function tweenParallax(
  state: ParallaxState,
  carouselApi: EmblaCarouselType,
  eventName?: EmblaEventType
) {
  const context = parallaxContext(carouselApi, eventName);
  carouselApi.scrollSnapList().forEach((scrollSnap, snapIndex) => {
    const slidesInSnap = context.engine.slideRegistry[snapIndex];
    slidesInSnap.forEach((slideIndex) => tweenSlide({ state, context, scrollSnap, slideIndex }));
  });
}

type TweenSlideOptions = {
  state: ParallaxState;
  context: ReturnType<typeof parallaxContext>;
  scrollSnap: number;
  slideIndex: number;
};

function tweenSlide(options: TweenSlideOptions) {
  if (options.context.isScrollEvent && !options.context.slidesInView.includes(options.slideIndex))
    return;
  const diffToTarget = slideDiff(options.context, options.scrollSnap, options.slideIndex);
  const translateValue = diffToTarget * (-1 * options.state.tweenFactor.current) * 100;
  const tweenNode = options.state.tweenNodes.current[options.slideIndex];
  if (tweenNode) tweenNode.style.transform = `translateX(${translateValue}%)`;
}

function parallaxContext(carouselApi: EmblaCarouselType, eventName?: EmblaEventType) {
  return {
    engine: carouselApi.internalEngine(),
    scrollProgress: carouselApi.scrollProgress(),
    slidesInView: carouselApi.slidesInView(),
    isScrollEvent: eventName === 'scroll',
  };
}

function slideDiff(
  context: ReturnType<typeof parallaxContext>,
  scrollSnap: number,
  slideIndex: number
) {
  if (!context.engine.options.loop) return scrollSnap - context.scrollProgress;
  return loopAdjustedDiff(context, scrollSnap, slideIndex);
}

function loopAdjustedDiff(
  context: ReturnType<typeof parallaxContext>,
  scrollSnap: number,
  slideIndex: number
) {
  let diffToTarget = scrollSnap - context.scrollProgress;
  context.engine.slideLooper.loopPoints.forEach((loopItem) => {
    const target = loopItem.target();
    if (slideIndex === loopItem.index && target !== 0) {
      diffToTarget =
        target < 0
          ? scrollSnap - (1 + context.scrollProgress)
          : scrollSnap + (1 - context.scrollProgress);
    }
  });
  return diffToTarget;
}
