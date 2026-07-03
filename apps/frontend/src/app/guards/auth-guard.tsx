'use client';

import { useEffect } from 'react';

import { paths } from 'src/shared/routes/paths';
import { SplashScreen } from 'src/shared/ui/loading-screen';
import { useRouter, usePathname } from 'src/shared/routes/hooks';

import { useAuthContext } from 'src/entities/session';

import { useMinimumChecking } from './use-minimum-checking';

// ----------------------------------------------------------------------

type AuthGuardProps = {
  children: React.ReactNode;
};

export function AuthGuard({ children }: AuthGuardProps) {
  const router = useRouter();
  const pathname = usePathname();

  const { authenticated, loading } = useAuthContext();

  const { isChecking, finishChecking } = useMinimumChecking();

  const createRedirectPath = (currentPath: string) => {
    const queryString = new URLSearchParams(pathname ? { returnTo: pathname } : {}).toString();
    return `${currentPath}?${queryString}`;
  };

  const checkPermissions = async (): Promise<void> => {
    if (loading) {
      return;
    }

    if (!authenticated) {
      const redirectPath = createRedirectPath(paths.auth.jwt.signIn);

      router.replace(redirectPath);

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
