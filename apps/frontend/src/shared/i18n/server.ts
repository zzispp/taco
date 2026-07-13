import type { Metadata } from 'next';
import type { I18nNamespace } from './types';
import type { LangCode } from './locales-config';

import { cache } from 'react';
import { headers } from 'next/headers';
import { createInstance } from 'i18next';
import acceptLanguage from 'accept-language';
import { initReactI18next } from 'react-i18next/initReactI18next';

import { formatDashboardDocumentTitle } from './document-title-format';
import {
  defaultNS,
  i18nOptions,
  fallbackLng,
  supportedLngs,
  i18nResourceLoader,
} from './locales-config';

// ----------------------------------------------------------------------

/**
 * Internationalization configuration for Next.js server-side.
 *
 * Server-side language detection uses the request Accept-Language header.
 * User-selected language persistence is handled on the client via localStorage.
 */

acceptLanguage.languages([...supportedLngs]);

function normalizeLanguage(value?: string | null): LangCode | undefined {
  if (!value) {
    return undefined;
  }

  const lower = value.toLowerCase().replace('_', '-');

  if (
    lower === 'tw' ||
    lower.startsWith('zh-tw') ||
    lower.startsWith('zh-hk') ||
    lower.startsWith('zh-hant')
  ) {
    return 'tw';
  }

  if (
    lower === 'cn' ||
    lower === 'zh' ||
    lower.startsWith('zh-cn') ||
    lower.startsWith('zh-hans')
  ) {
    return 'cn';
  }

  if (lower === 'en' || lower.startsWith('en-')) {
    return 'en';
  }

  return undefined;
}

function detectHeaderLanguage(header?: string | null): LangCode | undefined {
  if (!header) {
    return undefined;
  }

  return header
    .split(',')
    .map((part) => part.split(';')[0]?.trim())
    .map(normalizeLanguage)
    .find(Boolean);
}

export async function detectLanguage() {
  const headerStore = await headers();
  const headerLang = headerStore.get('accept-language') ?? undefined;
  const matchedLang = headerLang ? acceptLanguage.get(headerLang) : undefined;
  const fromHeader =
    detectHeaderLanguage(headerLang) ??
    normalizeLanguage(typeof matchedLang === 'string' ? matchedLang : undefined);

  const lang = fromHeader || fallbackLng;

  return lang as LangCode;
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
