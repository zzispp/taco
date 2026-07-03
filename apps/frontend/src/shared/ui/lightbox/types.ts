import type { LightboxExternalProps } from 'yet-another-react-lightbox';

// ----------------------------------------------------------------------

export type LightboxProps = LightboxExternalProps & {
  disableZoom?: boolean;
  disableVideo?: boolean;
  disableTotal?: boolean;
  disableCaptions?: boolean;
  disableSlideshow?: boolean;
  disableThumbnails?: boolean;
  disableFullscreen?: boolean;
  onGetCurrentIndex?: (index: number) => void;
};
