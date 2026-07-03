'use client';

import type { LangCode } from './locales-config';

import i18next from 'i18next';
import { useRef, useEffect } from 'react';
import { getStorage } from 'minimal-shared/utils';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import enAdmin from './langs/en/admin.json';
import cnAdmin from './langs/cn/admin.json';
import enCommon from './langs/en/common.json';
import cnCommon from './langs/cn/common.json';
import enNavbar from './langs/en/navbar.json';
import cnNavbar from './langs/cn/navbar.json';
import enMessages from './langs/en/messages.json';
import cnMessages from './langs/cn/messages.json';
import { i18nOptions, fallbackLng, storageConfig } from './locales-config';

// ----------------------------------------------------------------------

/**
 * Initialize i18next
 */
i18next.use(initReactI18next).init({
  ...i18nOptions(fallbackLng),
  ns: ['common', 'messages', 'admin', 'navbar'],
  resources: {
    cn: { admin: cnAdmin, common: cnCommon, messages: cnMessages, navbar: cnNavbar },
    en: { admin: enAdmin, common: enCommon, messages: enMessages, navbar: enNavbar },
  },
});

// ----------------------------------------------------------------------

type I18nProviderProps = {
  lang?: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  const mounted = useRef(false);
  const initialLang = lang ?? fallbackLng;

  if (!mounted.current && i18next.language !== initialLang) {
    i18next.changeLanguage(initialLang);
  }

  useEffect(() => {
    mounted.current = true;

    const storedLang = normalizeDetectedLanguage(getStorage(storageConfig.localStorage.key));
    const nextLang = storedLang ?? lang ?? fallbackLng;

    if (i18next.language !== nextLang) {
      i18next.changeLanguage(nextLang);
    }
  }, [lang]);

  return <Provider i18n={i18next}>{children}</Provider>;
}

function normalizeDetectedLanguage(lang?: string | null): LangCode | undefined {
  if (!lang) {
    return undefined;
  }

  const lower = lang.toLowerCase();

  if (lower === 'cn' || lower.startsWith('zh')) {
    return 'cn';
  }

  if (lower === 'en' || lower.startsWith('en')) {
    return 'en';
  }

  return undefined;
}
