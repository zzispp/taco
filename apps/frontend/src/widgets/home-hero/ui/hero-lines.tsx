import type { Variants } from 'framer-motion';

import { m } from 'framer-motion';

const ANIMATION_BASE_DELAY = 1;
const ANIMATION_DELAY_STEP = 0.5;

const drawLineX: Variants = {
  hidden: { x2: 0, strokeOpacity: 0 },
  visible: (index: number) => lineVariant(index, { x2: '100%' }),
};

const drawLineY: Variants = {
  hidden: { y2: 0, strokeOpacity: 0 },
  visible: (index: number) => lineVariant(index, { y2: '100%' }),
};

export function Lines({ strokeCount }: { strokeCount: number }) {
  return (
    <>
      {lineIndexes(strokeCount).map((index) => (
        <HorizontalLine key={`x-${index}`} index={index} strokeCount={strokeCount} />
      ))}
      {lineIndexes(strokeCount).map((index) => (
        <VerticalLine key={`y-${index}`} index={index} strokeCount={strokeCount} />
      ))}
    </>
  );
}

function HorizontalLine({ index, strokeCount }: LineProps) {
  return (
    <m.line
      x1="0"
      x2="100%"
      y1="50%"
      y2="50%"
      variants={drawLineX}
      style={lineStyle(translateByIndex('Y', index, strokeCount))}
    />
  );
}

function VerticalLine({ index, strokeCount }: LineProps) {
  return (
    <m.line
      x1="50%"
      x2="50%"
      y1="0%"
      y2="100%"
      variants={drawLineY}
      style={lineStyle(translateByIndex('X', index, strokeCount))}
    />
  );
}

type LineProps = {
  index: number;
  strokeCount: number;
};

function lineIndexes(strokeCount: number) {
  return Array.from({ length: strokeCount }, (_, index) => index);
}

function lineVariant(index: number, target: Record<string, string>) {
  const delay = ANIMATION_BASE_DELAY + index * ANIMATION_DELAY_STEP;
  return {
    ...target,
    strokeOpacity: 1,
    transition: {
      strokeOpacity: { delay, duration: 0.01 },
      [Object.keys(target)[0]]: { delay, bounce: 0, duration: 1.5 },
    },
  };
}

function translateByIndex(axis: 'X' | 'Y', index: number, strokeCount: number) {
  const half = strokeCount / 2;
  const distanceIndex = half > index ? index : strokeCount - (index + 1);
  const sign = half > index ? ' * -1' : '';
  return `translate${axis}(calc(((${distanceIndex} * var(--stroke-spacing)) + var(--stroke-spacing) / 2)${sign}))`;
}

function lineStyle(transform: string) {
  return {
    transform,
    stroke: 'var(--hero-line-stroke-color)',
    strokeDasharray: 'var(--stroke-dasharray)',
    strokeWidth: 'var(--hero-line-stroke-width)',
  };
}
