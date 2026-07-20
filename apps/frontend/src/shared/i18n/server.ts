import type { Metadata } from 'next';
import type { I18nNamespace } from './types';
import type { LangCode } from './locales-config';

import { createInstance } from 'i18next';
import { initReactI18next } from 'react-i18next/initReactI18next';

import { formatDashboardDocumentTitle } from './document-title-format';
import { defaultNS, i18nOptions, i18nResourceLoader } from './locales-config';

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

export async function getServerTranslations(
  lang: LangCode,
  namespace: I18nNamespace = defaultNS,
  options: Options = {}
) {
  const i18nextInstance = await initServerI18next(lang, namespace);

  return {
    t: i18nextInstance.getFixedT(lang, namespace, options.keyPrefix),
    i18n: i18nextInstance,
  };
}

export async function getDashboardPageMetadata(
  lang: LangCode,
  titleKey: string
): Promise<Metadata> {
  const { t } = await getServerTranslations(lang, 'admin');

  return {
    title: formatDashboardDocumentTitle(t(titleKey), t('nav.dashboard')),
  };
}
