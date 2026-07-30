'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

import { useSiteDisplay } from 'src/shared/config/site-display-context';

import { Logo } from './logo';

const SITE_TEXT_MAX_WIDTH = { xs: '44vw', sm: 200 } as const;

export type SiteBrandProps = Omit<BoxProps, 'children'> & {
  logoHref?: string;
  showSubtitle?: boolean;
};

export function SiteBrand({ sx, logoHref, showSubtitle = false, ...other }: SiteBrandProps) {
  const { siteName, siteSubtitle } = useSiteDisplay();

  return (
    <Box
      sx={[
        { display: 'flex', minWidth: 0, alignItems: 'center', gap: 1.25 },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <Logo href={logoHref} />
      <BrandText siteName={siteName} siteSubtitle={showSubtitle ? siteSubtitle : undefined} />
    </Box>
  );
}

type BrandTextProps = {
  siteName: string;
  siteSubtitle?: string;
};

function BrandText({ siteName, siteSubtitle }: BrandTextProps) {
  return (
    <Box
      sx={{
        minWidth: 0,
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'center',
        maxWidth: SITE_TEXT_MAX_WIDTH,
      }}
    >
      <Typography
        noWrap
        component="span"
        title={siteName}
        sx={{ fontSize: '1.125rem', fontWeight: 700, lineHeight: 1.1, letterSpacing: 0 }}
      >
        {siteName}
      </Typography>
      {siteSubtitle && (
        <Typography
          noWrap
          component="span"
          title={siteSubtitle}
          variant="caption"
          sx={{
            mt: 0.5,
            color: 'text.secondary',
            fontWeight: 500,
            lineHeight: 1,
            letterSpacing: 0,
          }}
        >
          {siteSubtitle}
        </Typography>
      )}
    </Box>
  );
}
