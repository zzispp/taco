'use client';

import type { LangCode } from './locales-config';

import { useEffect } from 'react';
import { type i18n, createInstance } from 'i18next';
import { initReactI18next, I18nextProvider as Provider } from 'react-i18next';

import { I18N_NAMESPACES } from './types';
import enAudit from './langs/en/audit.json';
import cnAudit from './langs/cn/audit.json';
import twAudit from './langs/tw/audit.json';
import enSetup from './langs/en/setup.json';
import cnSetup from './langs/cn/setup.json';
import twSetup from './langs/tw/setup.json';
import enCommon from './langs/en/common.json';
import cnCommon from './langs/cn/common.json';
import twCommon from './langs/tw/common.json';
import enNavbar from './langs/en/navbar.json';
import cnNavbar from './langs/cn/navbar.json';
import twNavbar from './langs/tw/navbar.json';
import { i18nOptions } from './locales-config';
import enMessages from './langs/en/messages.json';
import cnMessages from './langs/cn/messages.json';
import twMessages from './langs/tw/messages.json';
import enSystemLog from './langs/en/systemLog.json';
import cnSystemLog from './langs/cn/systemLog.json';
import twSystemLog from './langs/tw/systemLog.json';
import enScheduler from './langs/en/scheduler.json';
import cnScheduler from './langs/cn/scheduler.json';
import twScheduler from './langs/tw/scheduler.json';
import { updateDocumentLanguage } from './language';
import { staticAdminResources } from './admin-static-resources';

// ----------------------------------------------------------------------

const resources = {
  cn: {
    admin: staticAdminResources.cn,
    common: cnCommon,
    messages: cnMessages,
    navbar: cnNavbar,
    scheduler: cnScheduler,
    setup: cnSetup,
    audit: cnAudit,
    systemLog: cnSystemLog,
  },
  en: {
    admin: staticAdminResources.en,
    common: enCommon,
    messages: enMessages,
    navbar: enNavbar,
    scheduler: enScheduler,
    setup: enSetup,
    audit: enAudit,
    systemLog: enSystemLog,
  },
  tw: {
    admin: staticAdminResources.tw,
    common: twCommon,
    messages: twMessages,
    navbar: twNavbar,
    scheduler: twScheduler,
    setup: twSetup,
    audit: twAudit,
    systemLog: twSystemLog,
  },
};

const I18N_INSTANCES: Readonly<Record<LangCode, i18n>> = Object.freeze({
  cn: createI18n('cn'),
  en: createI18n('en'),
  tw: createI18n('tw'),
});

// ----------------------------------------------------------------------

type I18nProviderProps = {
  lang: LangCode;
  children: React.ReactNode;
};

export function I18nProvider({ lang, children }: I18nProviderProps) {
  useEffect(() => {
    updateDocumentLanguage(document.documentElement, lang);
  }, [lang]);

  return <Provider i18n={I18N_INSTANCES[lang]}>{children}</Provider>;
}

function createI18n(lang: LangCode) {
  const instance = createInstance();
  instance.use(initReactI18next);
  void instance.init({ ...i18nOptions(lang), initAsync: false, ns: I18N_NAMESPACES, resources });
  return instance;
}
