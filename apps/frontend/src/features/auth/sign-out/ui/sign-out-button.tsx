import type { ButtonProps } from '@mui/material/Button';

import { useCallback } from 'react';

import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { useRouter } from 'src/shared/routes/hooks';

import { useAuthContext } from 'src/entities/session';

import { signOut } from 'src/features/auth';

// ----------------------------------------------------------------------

type Props = ButtonProps & {
  onClose?: () => void;
};

export function SignOutButton({ onClose, sx, ...other }: Props) {
  const router = useRouter();

  const { checkUserSession } = useAuthContext();

  const handleLogout = useCallback(async () => {
    try {
      await signOut();
      await checkUserSession?.();

      onClose?.();
      router.refresh();
    } catch (error) {
      console.error(error);
      toast.error('Unable to logout!');
    }
  }, [checkUserSession, onClose, router]);

  return (
    <Button
      fullWidth
      variant="soft"
      size="large"
      color="error"
      onClick={handleLogout}
      sx={sx}
      {...other}
    >
      Logout
    </Button>
  );
}
