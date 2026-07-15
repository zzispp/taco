'use client';

import { useEffect } from 'react';

import { useRouter } from 'src/shared/routes/hooks';
import { SplashScreen } from 'src/shared/ui/loading-screen';

import { useAuthContext } from 'src/entities/session';

import { resolveRootTarget } from '../model/resolve-root-target';

export function RootRedirectPage() {
  const router = useRouter();
  const { loading, authenticated } = useAuthContext();
  const target = resolveRootTarget({ loading, authenticated });

  useEffect(() => {
    if (target) router.replace(target);
  }, [router, target]);

  return <SplashScreen />;
}
