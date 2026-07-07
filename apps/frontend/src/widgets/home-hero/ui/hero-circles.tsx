import type { Variants } from 'framer-motion';

import { m } from 'framer-motion';

const ANIMATION_BASE_DELAY = 1;
const ANIMATION_DELAY_STEP = 0.5;

const drawCircle: Variants = {
  hidden: { opacity: 0 },
  visible: (index: number) => ({
    opacity: 1,
    transition: { opacity: { delay: ANIMATION_BASE_DELAY + index * ANIMATION_DELAY_STEP, duration: 0.01 } },
  }),
};

export function Circles() {
  return (
    <>
      <CirclePath transform="translate(calc(50% - 480px), calc(50% - 80px))" />
      <CirclePath transform="translate(calc(50% + 400px), calc(50% + 80px))" />
      <m.circle
        cx="50%"
        cy="50%"
        fill="var(--hero-circle-stroke-color)"
        style={{ transform: 'translate(calc(0% - 200px), calc(0% + 200px))' }}
        initial={{ r: 0 }}
        animate={{ r: 5 }}
      />
    </>
  );
}

function CirclePath({ transform }: { transform: string }) {
  return (
    <m.path
      variants={drawCircle}
      d="M1 41C1 63.0914 18.9086 81 41 81C63.0914 81 81 63.0914 81 41C81 18.9086 63.0914 1 41 1"
      style={{
        transform,
        strokeDasharray: 'var(--stroke-dasharray)',
        stroke: 'var(--hero-circle-stroke-color)',
        strokeWidth: 'var(--hero-circle-stroke-width)',
      }}
    />
  );
}
