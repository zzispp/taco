'use client';

import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { Snackbar } from 'src/shared/ui/snackbar';
import { LocalizationProvider } from 'src/shared/i18n';
import { ProgressBar } from 'src/shared/ui/progress-bar';
import { I18nProvider } from 'src/shared/i18n/i18n-provider';
import { themeConfig, ThemeProvider } from 'src/shared/theme';
import { MotionLazy } from 'src/shared/ui/animate/motion-lazy';
import { SettingsDrawer, defaultSettings, SettingsProvider } from 'src/shared/ui/settings';

import { ApplicationSetupGate } from 'src/app/installation/application-setup-gate';

import { AuthProvider } from './auth-provider';
import { AppSettingsProvider } from './settings-provider';

type ApplicationProvidersProps = Readonly<{
  children: React.ReactNode;
}>;

export function ApplicationProviders({ children }: ApplicationProvidersProps) {
  return (
    <I18nProvider>
      <SettingsProvider defaultSettings={defaultSettings}>
        <ApplicationSetupGate>
          <InstalledApplicationProviders>{children}</InstalledApplicationProviders>
        </ApplicationSetupGate>
      </SettingsProvider>
    </I18nProvider>
  );
}

function InstalledApplicationProviders({ children }: ApplicationProvidersProps) {
  return (
    <AuthProvider>
      <AppSettingsProvider>
        <LocalizationProvider>
          <AppRouterCacheProvider options={{ key: 'css' }}>
            <ThemeProvider
              modeStorageKey={themeConfig.modeStorageKey}
              defaultMode={themeConfig.defaultMode}
            >
              <MotionLazy>
                <Snackbar />
                <ProgressBar />
                <SettingsDrawer />
                {children}
              </MotionLazy>
            </ThemeProvider>
          </AppRouterCacheProvider>
        </LocalizationProvider>
      </AppSettingsProvider>
    </AuthProvider>
  );
}
