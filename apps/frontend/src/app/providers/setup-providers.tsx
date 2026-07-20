'use client';

import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { Snackbar } from 'src/shared/ui/snackbar';
import { LocalizationProvider } from 'src/shared/i18n';
import { ProgressBar } from 'src/shared/ui/progress-bar';
import { themeConfig, ThemeProvider } from 'src/shared/theme';
import { MotionLazy } from 'src/shared/ui/animate/motion-lazy';
import { defaultSettings, SettingsProvider } from 'src/shared/ui/settings';

type SetupProvidersProps = Readonly<{
  children: React.ReactNode;
}>;

export function SetupProviders({ children }: SetupProvidersProps) {
  return (
    <SettingsProvider defaultSettings={defaultSettings}>
      <LocalizationProvider>
        <AppRouterCacheProvider options={{ key: 'css' }}>
          <ThemeProvider
            modeStorageKey={themeConfig.modeStorageKey}
            defaultMode={themeConfig.defaultMode}
          >
            <MotionLazy>
              <Snackbar />
              <ProgressBar />
              {children}
            </MotionLazy>
          </ThemeProvider>
        </AppRouterCacheProvider>
      </LocalizationProvider>
    </SettingsProvider>
  );
}
