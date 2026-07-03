'use client';

import { useEffect } from 'react';
import { safeReturnUrl } from 'minimal-shared/utils';

import { CONFIG } from 'src/shared/config';
import { SplashScreen } from 'src/shared/ui/loading-screen';
import { useRouter, useSearchParams } from 'src/shared/routes/hooks';

import { useAuthContext } from 'src/entities/session';

import { useMinimumChecking } from './use-minimum-checking';

// ----------------------------------------------------------------------

type GuestGuardProps = {
  children: React.ReactNode;
};

export function GuestGuard({ children }: GuestGuardProps) {
  const router = useRouter();

  const { loading, authenticated } = useAuthContext();

  const { isChecking, finishChecking } = useMinimumChecking();

  const searchParams = useSearchParams();
  const redirectUrl = safeReturnUrl(searchParams.get('returnTo'), CONFIG.auth.redirectPath);

  const checkPermissions = async (): Promise<void> => {
    if (loading) {
      return;
    }

    if (authenticated) {
      router.replace(redirectUrl);
      return;
    }

    finishChecking();
  };

  useEffect(() => {
    checkPermissions();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [authenticated, loading]);

  if (isChecking) {
    return <SplashScreen />;
  }

  return <>{children}</>;
}
