import type { Metadata } from 'next';
import type { I18nNamespace } from './types';
import type { LangCode } from './locales-config';

import { cache } from 'react';
import { createInstance } from 'i18next';
import { initReactI18next } from 'react-i18next/initReactI18next';

import { formatDashboardDocumentTitle } from './document-title-format';
import { defaultNS, i18nOptions, fallbackLng, i18nResourceLoader } from './locales-config';

// ----------------------------------------------------------------------

/**
 * Internationalization configuration for Next.js server-side.
 *
 * Static exports have no request-time language context. The client restores its
 * persisted choice after hydration, while build-time metadata uses the stable
 * fallback locale.
 */
export function detectLanguage(): LangCode {
  return fallbackLng;
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
    const lang = detectLanguage();
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
