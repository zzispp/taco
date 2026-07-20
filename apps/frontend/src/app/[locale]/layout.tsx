import { notFound } from 'next/navigation';

import InitColorSchemeScript from '@mui/material/InitColorSchemeScript';

import { isLangCode } from 'src/shared/routes/locale-path';
import { themeConfig } from 'src/shared/theme/theme-config';
import { I18nProvider } from 'src/shared/i18n/i18n-provider';
import { toDocumentLanguage } from 'src/shared/i18n/language';
import { supportedLngs } from 'src/shared/i18n/locales-config';

export const dynamicParams = false;

export function generateStaticParams() {
  return supportedLngs.map((locale) => ({ locale }));
}

type LocaleLayoutProps = Readonly<{
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}>;

export default async function LocaleLayout({ children, params }: LocaleLayoutProps) {
  const { locale } = await params;

  if (!isLangCode(locale)) notFound();

  return (
    <html lang={toDocumentLanguage(locale)} dir="ltr" suppressHydrationWarning>
      <body>
        <InitColorSchemeScript
          modeStorageKey={themeConfig.modeStorageKey}
          attribute={themeConfig.cssVariables.colorSchemeSelector}
          defaultMode={themeConfig.defaultMode}
        />
        <I18nProvider lang={locale}>{children}</I18nProvider>
      </body>
    </html>
  );
}
