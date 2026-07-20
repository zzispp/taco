import type { InitOptions } from 'i18next';
import type { Theme, Components } from '@mui/material/styles';

import resourcesToBackend from 'i18next-resources-to-backend';

// MUI Core Locales
import { zhCN as zhCNCore, zhTW as zhTWCore } from '@mui/material/locale';
// MUI Date Pickers Locales
import { enUS as enUSDate, zhCN as zhCNDate, zhTW as zhTWDate } from '@mui/x-date-pickers/locales';
// MUI Data Grid Locales
import {
  enUS as enUSDataGrid,
  zhCN as zhCNDataGrid,
  zhTW as zhTWDataGrid,
} from '@mui/x-data-grid/locales';

import { mergeAdminResources } from './admin-resources';
import localeContract from '../../../../../locale-contract.json';

// ----------------------------------------------------------------------

type LocaleContract = Readonly<{
  defaultLocale: string;
  locales: readonly Readonly<{
    code: string;
    documentLanguage: string;
    backendLanguage: string;
  }>[];
}>;

const contract = localeContract as LocaleContract;

export type LangCode = 'cn' | 'en' | 'tw';
export const supportedLngs = contract.locales.map(({ code }) => localeCode(code));

export const fallbackLng = localeCode(contract.defaultLocale);
export const defaultNS = 'common';

/**
 * @countryCode https://flagcdn.com/en/codes.json
 * @adapterLocale https://github.com/iamkun/dayjs/tree/master/src/locale
 * @numberFormat https://simplelocalize.io/data/locales/
 */

export type LangOption = {
  value: LangCode;
  label: string;
  countryCode: string;
  documentLanguage: string;
  backendLanguage: string;
  adapterLocale?: string;
  numberFormat: { code: string; currency: string };
  systemValue?: { components: Components<Theme> };
};

export const allLangs: LangOption[] = [
  {
    value: 'en',
    label: 'English',
    countryCode: 'GB',
    documentLanguage: contractLocale('en').documentLanguage,
    backendLanguage: contractLocale('en').backendLanguage,
    adapterLocale: 'en',
    numberFormat: { code: 'en-US', currency: 'USD' },
    systemValue: {
      components: { ...enUSDate.components, ...enUSDataGrid.components },
    },
  },
  {
    value: 'cn',
    label: 'Chinese',
    countryCode: 'CN',
    documentLanguage: contractLocale('cn').documentLanguage,
    backendLanguage: contractLocale('cn').backendLanguage,
    adapterLocale: 'zh-cn',
    numberFormat: { code: 'zh-CN', currency: 'CNY' },
    systemValue: {
      components: { ...zhCNCore.components, ...zhCNDate.components, ...zhCNDataGrid.components },
    },
  },
  {
    value: 'tw',
    label: 'Traditional Chinese',
    countryCode: 'TW',
    documentLanguage: contractLocale('tw').documentLanguage,
    backendLanguage: contractLocale('tw').backendLanguage,
    adapterLocale: 'zh-tw',
    numberFormat: { code: 'zh-TW', currency: 'TWD' },
    systemValue: {
      components: { ...zhTWCore.components, ...zhTWDate.components, ...zhTWDataGrid.components },
    },
  },
];

// ----------------------------------------------------------------------

export const i18nResourceLoader = resourcesToBackend(async (lang: LangCode, namespace: string) => {
  if (namespace === 'admin') {
    const [base, navigation, dashboard, accessControl, profile, onlineSessions, notice] =
      await Promise.all([
        import(`./langs/${lang}/admin.json`),
        import(`./langs/${lang}/admin-navigation.json`),
        import(`./langs/${lang}/admin-dashboard.json`),
        import(`./langs/${lang}/admin-access-control.json`),
        import(`./langs/${lang}/admin-profile.json`),
        import(`./langs/${lang}/admin-online-sessions.json`),
        import(`./langs/${lang}/admin-notice.json`),
      ]);
    return mergeAdminResources(
      base.default,
      navigation.default,
      dashboard.default,
      accessControl.default,
      profile.default,
      onlineSessions.default,
      notice.default
    );
  }
  const resource = await import(`./langs/${lang}/${namespace}.json`);
  return resource.default;
});

export function i18nOptions(lang = fallbackLng, namespace = defaultNS): InitOptions {
  return {
    // debug: true,
    supportedLngs,
    fallbackLng,
    lng: lang,
    /********/
    fallbackNS: defaultNS,
    defaultNS,
    ns: namespace,
  };
}

export function getCurrentLang(lang?: string): LangOption {
  const fallbackLang = allLangs.find((l) => l.value === fallbackLng) ?? allLangs[0];

  if (!lang) {
    return fallbackLang;
  }

  return allLangs.find((l) => l.value === lang) ?? fallbackLang;
}

function contractLocale(code: LangCode) {
  const locale = contract.locales.find((entry) => entry.code === code);
  if (!locale) {
    throw new Error(`Locale contract is missing language: ${code}`);
  }
  return locale;
}

function localeCode(value: string): LangCode {
  if (value === 'cn' || value === 'en' || value === 'tw') {
    return value;
  }
  throw new Error(`Locale contract has unsupported language: ${value}`);
}
