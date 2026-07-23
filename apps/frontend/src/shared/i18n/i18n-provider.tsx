'use client';

import type { LangCode } from './locales-config';

import { use, useEffect } from 'react';
import { type i18n, createInstance } from 'i18next';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import { I18N_NAMESPACES } from './types';
import { updateDocumentLanguage } from './language';
import { preloadLocaleRuntime } from './locale-runtime';
import { i18nOptions, i18nResourceLoader } from './locales-config';

const i18nInstances = new Map<LangCode, Promise<i18n>>();

type I18nProviderProps = {
  lang: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  const i18nInstance = use(loadI18nInstance(lang));

  useEffect(() => {
    updateDocumentLanguage(document.documentElement, lang);
  }, [lang]);

  return <Provider i18n={i18nInstance}>{children}</Provider>;
}

function loadI18nInstance(lang: LangCode): Promise<i18n> {
  const existingInstance = i18nInstances.get(lang);
  if (existingInstance) return existingInstance;

  const instance = createI18nInstance(lang);
  i18nInstances.set(lang, instance);
  return instance;
}

async function createI18nInstance(lang: LangCode): Promise<i18n> {
  const instance = createInstance();
  instance.use(initReactI18next).use(i18nResourceLoader);

  await Promise.all([
    instance.init({ ...i18nOptions(lang), ns: I18N_NAMESPACES }),
    preloadLocaleRuntime(lang),
  ]);

  return instance;
}
