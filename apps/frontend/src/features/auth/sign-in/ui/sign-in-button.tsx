import type { ButtonProps } from '@mui/material/Button';

import Button from '@mui/material/Button';

import { paths } from 'src/shared/routes/paths';
import { RouterLink } from 'src/shared/routes/components';

// ----------------------------------------------------------------------

export function SignInButton({ sx, ...other }: ButtonProps) {
  return (
    <Button
      component={RouterLink}
      href={paths.auth.jwt.signIn}
      variant="outlined"
      sx={sx}
      {...other}
    >
      Sign in
    </Button>
  );
}
