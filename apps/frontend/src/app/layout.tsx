import 'src/global.css';

import type { Metadata, Viewport } from 'next';

import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';

import { CONFIG } from 'src/shared/config';
import { primary as primaryColor } from 'src/shared/theme';
import { themeConfig } from 'src/shared/theme/theme-config';

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

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="zh-CN" dir="ltr" suppressHydrationWarning>
      <body>
        <InitColorSchemeScript
          modeStorageKey={themeConfig.modeStorageKey}
          attribute={themeConfig.cssVariables.colorSchemeSelector}
          defaultMode={themeConfig.defaultMode}
        />
        {children}
      </body>
    </html>
  );
}
