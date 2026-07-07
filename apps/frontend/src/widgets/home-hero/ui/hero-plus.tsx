import type { Variants } from 'framer-motion';

import { m } from 'framer-motion';

const ANIMATION_BASE_DELAY = 1;
const ANIMATION_DELAY_STEP = 0.5;

const drawPlus: Variants = {
  hidden: { opacity: 0, pathLength: 0 },
  visible: (index: number) => {
    const delay = ANIMATION_BASE_DELAY + index * ANIMATION_DELAY_STEP;
    return {
      opacity: 1,
      pathLength: 1,
      transition: {
        opacity: { delay, duration: 0.01 },
        pathLength: { delay, bounce: 0, duration: 1.5, type: 'spring' },
      },
    };
  },
};

export function PlusIcon() {
  return (
    <>
      <PlusPath transform="translate(calc(50% - 448px), calc(50% - 128px))" />
      <PlusPath transform="translate(calc(50% + 432px), calc(50% + 192px))" />
    </>
  );
}

function PlusPath({ transform }: { transform: string }) {
  return (
    <m.path
      variants={drawPlus}
      d="M8 0V16M16 8.08889H0"
      stroke="var(--hero-plus-stroke-color)"
      style={{ transform }}
    />
  );
}
