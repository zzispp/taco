'use client';

import type { Theme, SxProps } from '@mui/material/styles';
import type { LogoProps } from '../logo';

import { m } from 'framer-motion';
import { varAlpha } from 'minimal-shared/utils';

import { styled, keyframes } from '@mui/material/styles';

import { Logo } from '../logo';

// ----------------------------------------------------------------------

export type AnimateLogoProps = React.ComponentProps<'div'> & {
  sx?: SxProps<Theme>;
  logo?: React.ReactNode;
  slotProps?: {
    logo?: LogoProps;
  };
};

export function AnimateLogoZoom({ logo, slotProps, sx, ...other }: AnimateLogoProps) {
  return (
    <LogoZoomRoot sx={sx} {...other}>
      <LogoZoomCenter>
        {logo ?? (
          <Logo
            disabled
            {...slotProps?.logo}
            sx={[
              { width: 64, height: 64 },
              ...(Array.isArray(slotProps?.logo?.sx) ? slotProps.logo.sx : [slotProps?.logo?.sx]),
            ]}
          />
        )}
      </LogoZoomCenter>

      <LogoZoomPrimaryOutline />

      <LogoZoomSecondaryOutline />
    </LogoZoomRoot>
  );
}

const logoZoomCenter = keyframes`
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  33%, 66% {
    opacity: 0.48;
    transform: scale(0.9);
  }
`;

const logoZoomPrimaryOutline = keyframes`
  0%, 100% {
    opacity: 0.25;
    border-radius: 25%;
    transform: scale(1.6) rotate(270deg);
  }
  25%, 50% {
    opacity: 1;
    border-radius: 25%;
    transform: scale(1) rotate(0deg);
  }
  75% {
    opacity: 1;
    border-radius: 50%;
    transform: scale(1.6) rotate(270deg);
  }
`;

const logoZoomSecondaryOutline = keyframes`
  0%, 100% {
    opacity: 1;
    border-radius: 25%;
    transform: scale(1) rotate(0deg);
  }
  25%, 50%, 75% {
    opacity: 0.25;
    border-radius: 50%;
    transform: scale(1.2) rotate(270deg);
  }
`;

const LogoZoomRoot = styled('div')(() => ({
  width: 120,
  height: 120,
  alignItems: 'center',
  position: 'relative',
  display: 'inline-flex',
  justifyContent: 'center',
}));

const LogoZoomCenter = styled('span')({
  animation: `${logoZoomCenter} 3s ease-in-out infinite`,
});

const LogoZoomPrimaryOutline = styled('span')(({ theme }) => ({
  position: 'absolute',
  width: 'calc(100% - 20px)',
  height: 'calc(100% - 20px)',
  border: `solid 3px ${varAlpha(theme.vars.palette.primary.darkChannel, 0.24)}`,
  animation: `${logoZoomPrimaryOutline} 3.2s linear infinite`,
}));

const LogoZoomSecondaryOutline = styled('span')(({ theme }) => ({
  width: '100%',
  height: '100%',
  position: 'absolute',
  border: `solid 8px ${varAlpha(theme.vars.palette.primary.darkChannel, 0.24)}`,
  animation: `${logoZoomSecondaryOutline} 3.2s linear infinite`,
}));

// ----------------------------------------------------------------------

export function AnimateLogoRotate({ logo, sx, slotProps, ...other }: AnimateLogoProps) {
  return (
    <LogoRotateRoot sx={sx} {...other}>
      {logo ?? (
        <Logo
          {...slotProps?.logo}
          sx={[
            { zIndex: 9, width: 40, height: 40 },
            ...(Array.isArray(slotProps?.logo?.sx) ? slotProps.logo.sx : [slotProps?.logo?.sx]),
          ]}
        />
      )}

      <LogoRotateBackground
        animate={{ rotate: 360 }}
        transition={{ duration: 10, ease: 'linear', repeat: Infinity }}
      />
    </LogoRotateRoot>
  );
}

const LogoRotateRoot = styled('div')(() => ({
  width: 96,
  height: 96,
  alignItems: 'center',
  position: 'relative',
  display: 'inline-flex',
  justifyContent: 'center',
}));

const LogoRotateBackground = styled(m.span)(({ theme }) => ({
  width: '100%',
  height: '100%',
  opacity: 0.16,
  borderRadius: '50%',
  position: 'absolute',
  backgroundImage: `linear-gradient(135deg, transparent 50%, ${theme.vars.palette.primary.main} 100%)`,
  transition: theme.transitions.create(['opacity'], {
    easing: theme.transitions.easing.easeInOut,
    duration: theme.transitions.duration.shorter,
  }),
}));
