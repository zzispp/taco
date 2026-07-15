import type { Metadata } from 'next';
import type { I18nNamespace } from './types';
import type { LangCode } from './locales-config';

import { cache } from 'react';
import { headers } from 'next/headers';
import { createInstance } from 'i18next';
import { initReactI18next } from 'react-i18next/initReactI18next';

import { resolveAcceptedLanguage } from './language';
import { formatDashboardDocumentTitle } from './document-title-format';
import { defaultNS, i18nOptions, fallbackLng, i18nResourceLoader } from './locales-config';

// ----------------------------------------------------------------------

/**
 * Internationalization configuration for Next.js server-side.
 *
 * Server-side language detection uses the request Accept-Language header.
 * User-selected language persistence is handled on the client via localStorage.
 */

export async function detectLanguage() {
  const headerStore = await headers();
  const headerLang = headerStore.get('accept-language') ?? undefined;
  return resolveAcceptedLanguage(headerLang) ?? fallbackLng;
}

// ----------------------------------------------------------------------

export async function initServerI18next(lang: LangCode, namespace: I18nNamespace) {
  const i18nInstance = createInstance();
  const initOptions = i18nOptions(lang, namespace);

  await i18nInstance.use(initReactI18next).use(i18nResourceLoader).init(initOptions);

  return i18nInstance;
}

// ----------------------------------------------------------------------

type Options = Record<string, unknown> & {
  keyPrefix?: string;
};

export const getServerTranslations = cache(
  async (namespace: I18nNamespace = defaultNS, options: Options = {}) => {
    const lang = await detectLanguage();
    const i18nextInstance = await initServerI18next(lang, namespace);

    return {
      t: i18nextInstance.getFixedT(lang, namespace, options?.keyPrefix),
      i18n: i18nextInstance,
    };
  }
);

export async function getDashboardPageMetadata(titleKey: string): Promise<Metadata> {
  const { t } = await getServerTranslations('admin');

  return {
    title: formatDashboardDocumentTitle(t(titleKey), t('nav.dashboard')),
  };
}
