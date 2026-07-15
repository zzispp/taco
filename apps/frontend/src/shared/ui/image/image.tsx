'use client';

import type { UseInViewOptions } from 'framer-motion';
import type { Breakpoint } from '@mui/material/styles';
import type { EffectsType } from './styles';

import { useInView } from 'framer-motion';
import { mergeRefs, mergeClasses } from 'minimal-shared/utils';
import { useRef, useState, useCallback, startTransition } from 'react';

import { imageClasses } from './classes';
import { ImageImg, ImageRoot, ImageOverlay, ImagePlaceholder } from './styles';

// ----------------------------------------------------------------------

type PredefinedAspectRatio =
  '2/3' | '3/2' | '4/3' | '3/4' | '6/4' | '4/6' | '16/9' | '9/16' | '21/9' | '9/21' | '1/1';

type AspectRatioType = PredefinedAspectRatio | `${number}/${number}`;

export type ImageProps = React.ComponentProps<typeof ImageRoot> &
  Pick<React.ComponentProps<typeof ImageImg>, 'src' | 'alt'> & {
    delayTime?: number;
    onLoad?: () => void;
    effect?: EffectsType;
    visibleByDefault?: boolean;
    disablePlaceholder?: boolean;
    viewportOptions?: UseInViewOptions;
    ratio?: AspectRatioType | Partial<Record<Breakpoint, AspectRatioType>>;
    slotProps?: {
      img?: Omit<React.ComponentProps<typeof ImageImg>, 'src' | 'alt'>;
      overlay?: React.ComponentProps<typeof ImageOverlay>;
      placeholder?: React.ComponentProps<typeof ImagePlaceholder>;
    };
  };

const DEFAULT_DELAY = 0;
const DEFAULT_EFFECT: EffectsType = {
  style: 'blur',
  duration: 300,
  disabled: false,
};

type ImageLoadingOptions = {
  delayTime: number;
  onLoad?: () => void;
  viewportOptions?: UseInViewOptions;
  visibleByDefault: boolean;
  disablePlaceholder?: boolean;
};

type ImageViewProps = Omit<
  ImageProps,
  'delayTime' | 'onLoad' | 'viewportOptions' | 'visibleByDefault' | 'disablePlaceholder'
> & {
  localRef: React.RefObject<HTMLSpanElement | null>;
  isLoaded: boolean;
  handleImageLoad: () => void;
  shouldRenderImage: boolean;
  showPlaceholder: boolean;
  visibleByDefault: boolean;
};

type ImageContentProps = Pick<ImageProps, 'src' | 'slotProps'> & {
  alt: string;
  onLoad: () => void;
  shouldRenderImage: boolean;
  showPlaceholder: boolean;
};

export function Image({
  delayTime = DEFAULT_DELAY,
  onLoad,
  viewportOptions,
  disablePlaceholder,
  visibleByDefault = false,
  ...viewProps
}: ImageProps) {
  const loading = useImageLoading({
    delayTime,
    onLoad,
    viewportOptions,
    visibleByDefault,
    disablePlaceholder,
  });

  return <ImageView {...viewProps} {...loading} visibleByDefault={visibleByDefault} />;
}

function ImageView({
  sx,
  src,
  ref,
  ratio,
  effect,
  alt = '',
  slotProps,
  className,
  localRef,
  isLoaded,
  handleImageLoad,
  shouldRenderImage,
  showPlaceholder,
  visibleByDefault,
  ...other
}: ImageViewProps) {
  const finalEffect = {
    ...DEFAULT_EFFECT,
    ...effect,
  };

  return (
    <ImageRoot
      ref={mergeRefs([localRef, ref])}
      effect={visibleByDefault || finalEffect.disabled ? undefined : finalEffect}
      className={mergeClasses([imageClasses.root, className], {
        [imageClasses.state.loaded]: !visibleByDefault && isLoaded,
      })}
      sx={[
        {
          '--aspect-ratio': ratio,
          ...(!!ratio && { width: 1 }),
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <ImageContent
        src={src}
        alt={alt}
        slotProps={slotProps}
        onLoad={handleImageLoad}
        showPlaceholder={showPlaceholder}
        shouldRenderImage={shouldRenderImage}
      />
    </ImageRoot>
  );
}

function useImageLoading(options: ImageLoadingOptions) {
  const { delayTime, onLoad, viewportOptions, visibleByDefault, disablePlaceholder } = options;
  const localRef = useRef<HTMLSpanElement>(null);
  const [isLoaded, setIsLoaded] = useState(false);
  const isInView = useInView(localRef, { once: true, ...viewportOptions });

  const handleImageLoad = useCallback(() => {
    const timer = setTimeout(() => {
      startTransition(() => {
        setIsLoaded(true);
        onLoad?.();
      });
    }, delayTime);

    return () => clearTimeout(timer);
  }, [delayTime, onLoad]);

  return {
    localRef,
    isLoaded,
    handleImageLoad,
    shouldRenderImage: visibleByDefault || isInView,
    showPlaceholder: !visibleByDefault && !isLoaded && !disablePlaceholder,
  };
}

function ImageContent({
  src,
  alt,
  slotProps,
  onLoad,
  showPlaceholder,
  shouldRenderImage,
}: ImageContentProps) {
  return (
    <>
      {slotProps?.overlay && (
        <ImageOverlay className={imageClasses.overlay} {...slotProps.overlay} />
      )}
      {showPlaceholder && (
        <ImagePlaceholder className={imageClasses.placeholder} {...slotProps?.placeholder} />
      )}
      {shouldRenderImage && (
        <ImageImg
          src={src}
          alt={alt}
          onLoad={onLoad}
          className={imageClasses.img}
          {...slotProps?.img}
        />
      )}
    </>
  );
}
