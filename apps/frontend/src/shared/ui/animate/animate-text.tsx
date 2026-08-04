'use client';

import type { RefObject } from 'react';
import type { Theme, SxProps } from '@mui/material/styles';
import type { TypographyProps } from '@mui/material/Typography';
import type { Variants, UseInViewOptions } from 'framer-motion';

import { useRef, useMemo, useEffect } from 'react';
import { mergeClasses } from 'minimal-shared/utils';
import { m, useInView, useAnimation } from 'framer-motion';

import { styled } from '@mui/material/styles';
import Typography from '@mui/material/Typography';

import { createClasses } from 'src/shared/theme/create-classes';

import { varFade, varContainer } from './variants';

// ----------------------------------------------------------------------

const DEFAULT_REPEAT_DELAY_MS = 100;

export const animateTextClasses = {
  root: createClasses('animate__text__root'),
  lines: createClasses('animate__text__lines'),
  line: createClasses('animate__text__line'),
  word: createClasses('animate__text__word'),
  char: createClasses('animate__text__char'),
  space: createClasses('animate__text__space'),
  srOnly: 'sr-only',
};

const srOnlyStyles: SxProps<Theme> = {
  p: 0,
  width: '1px',
  height: '1px',
  margin: '-1px',
  borderWidth: 0,
  overflow: 'hidden',
  position: 'absolute',
  whiteSpace: 'nowrap',
  clip: 'rect(0, 0, 0, 0)',
};

export type AnimateTextProps = TypographyProps & {
  variants?: Variants;
  repeatDelayMs?: number;
  textContent: string | string[];
  once?: UseInViewOptions['once'];
  amount?: UseInViewOptions['amount'];
};

type TextAnimationOptions = Pick<
  AnimateTextProps,
  'textContent' | 'once' | 'amount' | 'repeatDelayMs'
>;

type AnimatedTextLinesProps = {
  textLines: string[];
  textRef: RefObject<HTMLSpanElement | null>;
  variants?: Variants;
  animationControls: ReturnType<typeof useAnimation>;
};

function useTextAnimation({ textContent, once, amount, repeatDelayMs }: TextAnimationOptions) {
  const textRef = useRef<HTMLSpanElement>(null);
  const animationControls = useAnimation();
  const textLines = useMemo(
    () => (Array.isArray(textContent) ? textContent : [textContent]),
    [textContent]
  );
  const isInView = useInView(textRef, { once, amount });

  useEffect(() => {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    const startAnimation = async () => {
      await animationControls.start('initial');
      animationControls.start('animate');
    };

    if (isInView && repeatDelayMs) {
      timeout = setTimeout(startAnimation, repeatDelayMs);
    } else if (isInView) {
      animationControls.start('animate');
    } else {
      animationControls.start('initial');
    }

    return () => {
      if (timeout) {
        clearTimeout(timeout);
      }
    };
  }, [animationControls, isInView, repeatDelayMs]);

  return { textRef, animationControls, textLines };
}

export function AnimateText({
  sx,
  variants,
  className,
  textContent,
  once = true,
  amount = 1 / 3,
  component = 'p',
  repeatDelayMs = DEFAULT_REPEAT_DELAY_MS,
  ...other
}: AnimateTextProps) {
  const { textRef, animationControls, textLines } = useTextAnimation({
    textContent,
    once,
    amount,
    repeatDelayMs,
  });

  return (
    <Typography
      component={component}
      className={mergeClasses([animateTextClasses.root, className])}
      sx={[
        {
          p: 0,
          m: 0,
          [`& .${animateTextClasses.srOnly}`]: srOnlyStyles,
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <span className={animateTextClasses.srOnly}>{textLines.join(' ')}</span>
      <AnimatedTextLines
        textLines={textLines}
        textRef={textRef}
        variants={variants}
        animationControls={animationControls}
      />
    </Typography>
  );
}

function AnimatedTextLines({
  textLines,
  textRef,
  variants,
  animationControls,
}: AnimatedTextLinesProps) {
  return (
    <AnimatedTextContainer
      aria-hidden
      ref={textRef}
      initial="initial"
      animate={animationControls}
      exit="exit"
      variants={varContainer()}
      className={animateTextClasses.lines}
    >
      {textLines.map((line, lineIndex) => (
        <AnimatedTextLine
          key={`${line}-${lineIndex}`}
          line={line}
          lineIndex={lineIndex}
          variants={variants}
        />
      ))}
    </AnimatedTextContainer>
  );
}

function AnimatedTextLine({
  line,
  lineIndex,
  variants,
}: {
  line: string;
  lineIndex: number;
  variants?: Variants;
}) {
  const words = line.split(' ');
  const lastWord = words[words.length - 1];

  return (
    <TextLine data-index={lineIndex} className={animateTextClasses.line} sx={{ display: 'block' }}>
      {words.map((word, wordIndex) => (
        <AnimatedTextWord
          key={`${word}-${wordIndex}`}
          word={word}
          wordIndex={wordIndex}
          showSpace={lastWord !== word}
          variants={variants}
        />
      ))}
    </TextLine>
  );
}

function AnimatedTextWord({
  word,
  wordIndex,
  showSpace,
  variants,
}: {
  word: string;
  wordIndex: number;
  showSpace: boolean;
  variants?: Variants;
}) {
  return (
    <TextWord
      data-index={wordIndex}
      className={animateTextClasses.word}
      sx={{ display: 'inline-block' }}
    >
      {word.split('').map((char, charIndex) => (
        <AnimatedTextChar
          key={`${char}-${charIndex}`}
          variants={variants ?? varFade('in')}
          data-index={charIndex}
          className={animateTextClasses.char}
          sx={{ display: 'inline-block' }}
        >
          {char}
        </AnimatedTextChar>
      ))}
      {showSpace && (
        <TextWord className={animateTextClasses.space} sx={{ display: 'inline-block' }}>
          &nbsp;
        </TextWord>
      )}
    </TextWord>
  );
}

// ----------------------------------------------------------------------

const TextLine = styled('span')``;

const TextWord = styled('span')``;

const AnimatedTextContainer = styled(m.span)``;

const AnimatedTextChar = styled(m.span)``;
