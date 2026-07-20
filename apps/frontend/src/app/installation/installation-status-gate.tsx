'use client';

import type { InstallationState } from 'src/entities/installation';

import { useEffect } from 'react';

import { useTranslate } from 'src/shared/i18n';
import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';
import { SplashScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { useInstallationStatus } from 'src/entities/installation';

type InstallationStatusGateProps = Readonly<{
  children: React.ReactNode;
  expectedState: InstallationState;
  loadingFallback?: React.ReactNode;
}>;

export function InstallationStatusGate({
  children,
  expectedState,
  loadingFallback = <SplashScreen />,
}: InstallationStatusGateProps) {
  const router = useRouter();
  const probe = useInstallationStatus();
  const { t } = useTranslate('setup');
  const redirectPath = targetPath(probe.kind === 'ready' ? probe.state : undefined, expectedState);

  useEffect(() => {
    if (redirectPath) router.replace(redirectPath);
  }, [redirectPath, router]);

  if (probe.kind === 'loading' || redirectPath) {
    return loadingFallback;
  }
  if (probe.kind === 'failure') {
    return (
      <InstallationGateScreen
        title={t('status.probeFailedTitle')}
        detail={getErrorMessage(probe.error)}
        retryLabel={t('actions.retry')}
        onRetry={probe.retry}
      />
    );
  }

  return <>{children}</>;
}

function targetPath(state: InstallationState | undefined, expectedState: InstallationState) {
  if (!state || state === expectedState) return undefined;
  return state === 'setup' ? paths.setup.root : paths.auth.jwt.signIn;
}

type InstallationGateScreenProps = Readonly<{
  title: string;
  detail?: string;
  retryLabel?: string;
  onRetry?: () => void;
}>;

function InstallationGateScreen({
  title,
  detail,
  retryLabel,
  onRetry,
}: InstallationGateScreenProps) {
  return (
    <main style={screenStyle}>
      <section aria-live="polite" style={panelStyle}>
        <h1>{title}</h1>
        {detail ? <p>{detail}</p> : null}
        {onRetry && retryLabel ? (
          <button type="button" onClick={onRetry}>
            {retryLabel}
          </button>
        ) : null}
      </section>
    </main>
  );
}

const screenStyle: React.CSSProperties = {
  alignItems: 'center',
  display: 'flex',
  justifyContent: 'center',
  minHeight: '100vh',
  padding: '24px',
};

const panelStyle: React.CSSProperties = {
  maxWidth: '560px',
  textAlign: 'center',
};
