import type { ButtonProps } from '@mui/material/Button';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n';
import { paths } from 'src/shared/routes/paths';
import { RouterLink } from 'src/shared/routes/components';

// ----------------------------------------------------------------------

export function SignInButton({ sx, ...other }: ButtonProps) {
  const { t } = useTranslate('messages');

  return (
    <Button
      component={RouterLink}
      href={paths.auth.jwt.signIn}
      variant="outlined"
      sx={sx}
      {...other}
    >
      {t('auth.signIn.submit')}
    </Button>
  );
}
