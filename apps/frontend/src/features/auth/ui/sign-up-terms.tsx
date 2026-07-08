import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';

import { useTranslate } from 'src/shared/i18n';

// ----------------------------------------------------------------------

export function SignUpTerms({ sx, ...other }: BoxProps) {
  const { t } = useTranslate('messages');

  return (
    <Box
      component="span"
      sx={[
        () => ({
          mt: 3,
          display: 'block',
          textAlign: 'center',
          typography: 'caption',
          color: 'text.secondary',
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
      {...other}
    >
      {`${t('auth.signUp.termsPrefix')} `}
      <Link underline="always" color="text.primary">
        {t('auth.signUp.terms')}
      </Link>
      {` ${t('auth.signUp.termsConnector')} `}
      <Link underline="always" color="text.primary">
        {t('auth.signUp.privacy')}
      </Link>
      .
    </Box>
  );
}
