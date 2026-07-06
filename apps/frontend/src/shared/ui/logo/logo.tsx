'use client';

import type { LinkProps } from '@mui/material/Link';

import { mergeClasses } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import { styled } from '@mui/material/styles';

import { RouterLink } from 'src/shared/routes/components';
import { useSiteDisplay } from 'src/shared/config/site-display-context';

import { logoClasses } from './classes';

const SINGLE_LOGO_SIZE = { width: 40, height: 40 };
const FULL_LOGO_SIZE = { width: 102, height: 36 };

export type LogoProps = LinkProps & {
  isSingle?: boolean;
  disabled?: boolean;
};

export function Logo({
  sx,
  disabled,
  className,
  href = '/',
  isSingle = true,
  ...other
}: LogoProps) {
  const size = isSingle ? SINGLE_LOGO_SIZE : FULL_LOGO_SIZE;
  const { logoUrl, siteName } = useSiteDisplay();

  return (
    <LogoRoot
      component={RouterLink}
      href={href}
      aria-label={`${siteName} logo`}
      underline="none"
      className={mergeClasses([logoClasses.root, className])}
      sx={[
        { ...size, ...(disabled && { pointerEvents: 'none' }) },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Box
        component="img"
        alt={siteName}
        src={logoUrl}
        sx={{
          width: '100%',
          height: '100%',
          display: 'block',
          objectFit: 'contain',
          objectPosition: isSingle ? 'center' : 'left center',
        }}
      />
    </LogoRoot>
  );
}

const LogoRoot = styled(Link)(() => ({
  flexShrink: 0,
  color: 'transparent',
  display: 'inline-flex',
  verticalAlign: 'middle',
}));
