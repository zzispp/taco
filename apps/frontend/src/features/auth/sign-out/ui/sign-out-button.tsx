import type { ButtonProps } from '@mui/material/Button';

import { useCallback } from 'react';

import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { useRouter } from 'src/shared/routes/hooks';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useAuthContext } from 'src/entities/session';

import { signOut } from 'src/features/auth';

// ----------------------------------------------------------------------

type Props = ButtonProps & {
  onClose?: () => void;
};

export function SignOutButton({ onClose, sx, ...other }: Props) {
  const router = useRouter();
  const { t } = useTranslate('admin');

  const { checkUserSession } = useAuthContext();

  const handleLogout = useCallback(async () => {
    try {
      await signOut();
      await checkUserSession?.();

      onClose?.();
      router.refresh();
    } catch (error) {
      console.error(error);
      toast.error(t('profile.logoutFailed'));
    }
  }, [checkUserSession, onClose, router, t]);

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
      {t('profile.logout')}
    </Button>
  );
}
