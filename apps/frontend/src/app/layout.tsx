import 'src/global.css';

import type { Metadata, Viewport } from 'next';

import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';
import { AppRouterCacheProvider } from '@mui/material-nextjs/v15-appRouter';

import { CONFIG } from 'src/shared/config';
import { Snackbar } from 'src/shared/ui/snackbar';
import { LocalizationProvider } from 'src/shared/i18n';
import { detectLanguage } from 'src/shared/i18n/server';
import { ProgressBar } from 'src/shared/ui/progress-bar';
import { I18nProvider } from 'src/shared/i18n/i18n-provider';
import { MotionLazy } from 'src/shared/ui/animate/motion-lazy';
import { themeConfig, ThemeProvider, primary as primaryColor } from 'src/shared/theme';
import { SettingsDrawer, defaultSettings, SettingsProvider } from 'src/shared/ui/settings';

import { AuthProvider } from 'src/app/providers/auth-provider';

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: primaryColor.main,
};

export const metadata: Metadata = {
  icons: [
    {
      rel: 'icon',
      type: 'image/svg+xml',
      url: `${CONFIG.assetsDir}/logo/logo.svg`,
    },
  ],
};

// ----------------------------------------------------------------------

type RootLayoutProps = {
  children: React.ReactNode;
};

async function getAppConfig() {
  if (CONFIG.isStaticExport) {
    return {
      lang: 'en',
      i18nLang: undefined,
      dir: defaultSettings.direction,
    };
  } else {
    const lang = await detectLanguage();

    return {
      lang,
      i18nLang: lang,
      dir: defaultSettings.direction,
    };
  }
}

export default async function RootLayout({ children }: RootLayoutProps) {
  const appConfig = await getAppConfig();

  return (
    <html lang={appConfig.lang} dir={appConfig.dir} suppressHydrationWarning>
      <body>
        <InitColorSchemeScript
          modeStorageKey={themeConfig.modeStorageKey}
          attribute={themeConfig.cssVariables.colorSchemeSelector}
          defaultMode={themeConfig.defaultMode}
        />

        <I18nProvider lang={appConfig.i18nLang}>
          <AuthProvider>
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
                      <SettingsDrawer defaultSettings={defaultSettings} />
                      {children}
                    </MotionLazy>
                  </ThemeProvider>
                </AppRouterCacheProvider>
              </LocalizationProvider>
            </SettingsProvider>
          </AuthProvider>
        </I18nProvider>
      </body>
    </html>
  );
}
