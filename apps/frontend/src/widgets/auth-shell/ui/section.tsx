import type { BoxProps } from '@mui/material/Box';
import type { Theme, SxProps, Breakpoint } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

import { CONFIG } from 'src/shared/config';
import { useTranslate } from 'src/shared/i18n';
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
  const { t } = useTranslate('messages');
  const { siteName } = useSiteDisplay();
  const resolvedTitle = title ?? authSectionTitle({ variant, siteName, t });
  const resolvedSubtitle = subtitle ?? t('auth.section.subtitle', { siteName });

  return (
    <Box sx={createAuthSectionSx(layoutQuery, sx)} {...other}>
      <AuthSectionContent
        title={resolvedTitle}
        subtitle={resolvedSubtitle}
        imgUrl={imgUrl}
        illustrationAlt={t('auth.section.illustrationAlt')}
      />
    </Box>
  );
}

function createAuthSectionSx(layoutQuery: Breakpoint, sx?: SxProps<Theme>) {
  return [
    (theme: Theme) => ({
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
  ];
}

function AuthSectionContent({
  title,
  subtitle,
  imgUrl,
  illustrationAlt,
}: {
  title: string;
  subtitle?: string;
  imgUrl: string;
  illustrationAlt: string;
}) {
  return (
    <>
      <div>
        <Typography variant="h3" sx={{ textAlign: 'center' }}>
          {title}
        </Typography>
        {subtitle && (
          <Typography sx={{ color: 'text.secondary', textAlign: 'center', mt: 2 }}>
            {subtitle}
          </Typography>
        )}
      </div>
      <Box
        component="img"
        alt={illustrationAlt}
        src={imgUrl}
        sx={{ width: 1, aspectRatio: '4/3', objectFit: 'cover' }}
      />
    </>
  );
}

type AuthSectionTitleOptions = {
  siteName: string;
  t: ReturnType<typeof useTranslate>['t'];
  variant: AuthSectionVariant;
};

function authSectionTitle({ variant, siteName, t }: AuthSectionTitleOptions) {
  if (variant === 'sign-in') {
    return t('auth.section.signInTitle', { siteName });
  }

  if (variant === 'sign-up') {
    return t('auth.section.signUpTitle', { siteName });
  }

  return t('auth.section.defaultTitle');
}
