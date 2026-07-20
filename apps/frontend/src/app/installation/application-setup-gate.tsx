'use client';

import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { SplashScreen } from 'src/shared/ui/loading-screen';
import { themeConfig, ThemeProvider } from 'src/shared/theme';

import { InstallationStatusGate } from './installation-status-gate';

type ApplicationSetupGateProps = Readonly<{
  children: React.ReactNode;
}>;

export function ApplicationSetupGate({ children }: ApplicationSetupGateProps) {
  return (
    <InstallationStatusGate
      expectedState="installed"
      loadingFallback={<ApplicationSetupLoadingFallback />}
    >
      {children}
    </InstallationStatusGate>
  );
}

function ApplicationSetupLoadingFallback() {
  return (
    <AppRouterCacheProvider options={{ key: 'css' }}>
      <ThemeProvider
        modeStorageKey={themeConfig.modeStorageKey}
        defaultMode={themeConfig.defaultMode}
      >
        <SplashScreen />
      </ThemeProvider>
    </AppRouterCacheProvider>
  );
}
