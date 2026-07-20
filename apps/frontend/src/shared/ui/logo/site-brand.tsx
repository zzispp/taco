'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

import { useSiteDisplay } from 'src/shared/config/site-display-context';

import { Logo } from './logo';

const SITE_NAME_MAX_WIDTH = { xs: '40vw', sm: 360 } as const;

export type SiteBrandProps = Omit<BoxProps, 'children'> & {
  logoHref?: string;
};

export function SiteBrand({ sx, logoHref, ...other }: SiteBrandProps) {
  const { siteName } = useSiteDisplay();

  return (
    <Box
      sx={[
        { display: 'flex', minWidth: 0, alignItems: 'center', gap: 1 },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Logo href={logoHref} />
      <Typography
        noWrap
        component="span"
        title={siteName}
        variant="h4"
        sx={{ maxWidth: SITE_NAME_MAX_WIDTH }}
      >
        {siteName}
      </Typography>
    </Box>
  );
}
