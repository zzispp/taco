import type { BoxProps } from '@mui/material/Box';
import type { Breakpoint } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/shared/config';
import { useSiteDisplay } from 'src/shared/config/site-display-context';

// ----------------------------------------------------------------------

type AuthSectionVariant = 'default' | 'sign-in' | 'sign-up';

export type AuthSplitSectionProps = BoxProps & {
  title?: string;
  imgUrl?: string;
  subtitle?: string;
  variant?: AuthSectionVariant;
  layoutQuery?: Breakpoint;
};

export function AuthSplitSection({
  sx,
  layoutQuery = 'md',
  variant = 'default',
  title,
  imgUrl = `${CONFIG.assetsDir}/assets/illustrations/illustration-dashboard.webp`,
  subtitle,
  ...other
}: AuthSplitSectionProps) {
  const { siteName } = useSiteDisplay();
  const resolvedTitle = title ?? authSectionTitle(variant, siteName);
  const resolvedSubtitle =
    subtitle ??
    `${siteName} centralizes authentication, RBAC, API permissions, and menu governance.`;

  return (
    <Box
      sx={[
        (theme) => ({
          ...theme.mixins.bgGradient({
            images: [
              `linear-gradient(0deg, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.92)}, ${varAlpha(theme.vars.palette.background.defaultChannel, 0.92)})`,
              `url(${CONFIG.assetsDir}/assets/background/background-3-blur.webp)`,
            ],
          }),
          px: 3,
          pb: 3,
          width: 1,
          maxWidth: 480,
          display: 'none',
          position: 'relative',
          pt: 'var(--layout-header-desktop-height)',
          [theme.breakpoints.up(layoutQuery)]: {
            gap: 8,
            display: 'flex',
            alignItems: 'center',
            flexDirection: 'column',
            justifyContent: 'center',
          },
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      <div>
        <Typography variant="h3" sx={{ textAlign: 'center' }}>
          {resolvedTitle}
        </Typography>

        {resolvedSubtitle && (
          <Typography sx={{ color: 'text.secondary', textAlign: 'center', mt: 2 }}>
            {resolvedSubtitle}
          </Typography>
        )}
      </div>

      <Box
        component="img"
        alt="Dashboard illustration"
        src={imgUrl}
        sx={{ width: 1, aspectRatio: '4/3', objectFit: 'cover' }}
      />
    </Box>
  );
}

function authSectionTitle(variant: AuthSectionVariant, siteName: string) {
  if (variant === 'sign-in') {
    return `Resume your ${siteName} workspace`;
  }

  if (variant === 'sign-up') {
    return `Create your ${siteName} workspace`;
  }

  return 'Operate one backend control plane';
}
