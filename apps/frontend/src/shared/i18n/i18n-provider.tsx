'use client';

import type { LangCode } from './locales-config';

import i18next from 'i18next';
import { useRef, useEffect } from 'react';
import { getStorage } from 'minimal-shared/utils';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import { I18N_NAMESPACES } from './types';
import enAudit from './langs/en/audit.json';
import cnAudit from './langs/cn/audit.json';
import twAudit from './langs/tw/audit.json';
import enCommon from './langs/en/common.json';
import cnCommon from './langs/cn/common.json';
import twCommon from './langs/tw/common.json';
import enNavbar from './langs/en/navbar.json';
import cnNavbar from './langs/cn/navbar.json';
import twNavbar from './langs/tw/navbar.json';
import { normalizeLanguage } from './language';
import enMessages from './langs/en/messages.json';
import cnMessages from './langs/cn/messages.json';
import twMessages from './langs/tw/messages.json';
import enScheduler from './langs/en/scheduler.json';
import cnScheduler from './langs/cn/scheduler.json';
import twScheduler from './langs/tw/scheduler.json';
import { staticAdminResources } from './admin-static-resources';
import { i18nOptions, fallbackLng, storageConfig } from './locales-config';

// ----------------------------------------------------------------------

/**
 * Initialize i18next
 */
i18next.use(initReactI18next).init({
  ...i18nOptions(fallbackLng),
  ns: I18N_NAMESPACES,
  resources: {
    cn: {
      admin: staticAdminResources.cn,
      common: cnCommon,
      messages: cnMessages,
      navbar: cnNavbar,
      scheduler: cnScheduler,
      audit: cnAudit,
    },
    en: {
      admin: staticAdminResources.en,
      common: enCommon,
      messages: enMessages,
      navbar: enNavbar,
      scheduler: enScheduler,
      audit: enAudit,
    },
    tw: {
      admin: staticAdminResources.tw,
      common: twCommon,
      messages: twMessages,
      navbar: twNavbar,
      scheduler: twScheduler,
      audit: twAudit,
    },
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

    const storedLang = normalizeLanguage(getStorage(storageConfig.localStorage.key));
    const nextLang = storedLang ?? lang ?? fallbackLng;

    if (i18next.language !== nextLang) {
      i18next.changeLanguage(nextLang);
    }
  }, [lang]);

  return <Provider i18n={i18next}>{children}</Provider>;
}
