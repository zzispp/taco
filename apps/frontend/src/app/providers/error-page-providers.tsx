'use client';

import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { LocalizationProvider } from 'src/shared/i18n';
import { I18nProvider } from 'src/shared/i18n/i18n-provider';
import { fallbackLng } from 'src/shared/i18n/locales-config';
import { themeConfig, ThemeProvider } from 'src/shared/theme';
import { MotionLazy } from 'src/shared/ui/animate/motion-lazy';
import { SettingsDrawer, defaultSettings, SettingsProvider } from 'src/shared/ui/settings';

type ErrorPageProvidersProps = Readonly<{
  children: React.ReactNode;
}>;

export function ErrorPageProviders({ children }: ErrorPageProvidersProps) {
  return (
    <I18nProvider lang={fallbackLng}>
      <SettingsProvider defaultSettings={defaultSettings}>
        <LocalizationProvider>
          <AppRouterCacheProvider options={{ key: 'css' }}>
            <ThemeProvider modeStorageKey={themeConfig.modeStorageKey} defaultMode={themeConfig.defaultMode}>
              <MotionLazy>
                <SettingsDrawer />
                {children}
              </MotionLazy>
            </ThemeProvider>
          </AppRouterCacheProvider>
        </LocalizationProvider>
      </SettingsProvider>
    </I18nProvider>
  );
}
