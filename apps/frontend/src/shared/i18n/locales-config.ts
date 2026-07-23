import type { InitOptions } from 'i18next';

import resourcesToBackend from 'i18next-resources-to-backend';

import { mergeAdminResources } from './admin-resources';
import { I18N_NAMESPACES, type I18nNamespace } from './types';
import {
  type LocaleCode,
  defaultLocaleCode,
  supportedLocaleCodes,
  getLocaleContractEntry,
} from './locale-contract';

// ----------------------------------------------------------------------

export type LangCode = LocaleCode;
export const supportedLngs = supportedLocaleCodes;

export const fallbackLng = defaultLocaleCode;
export const defaultNS = 'common';

export type LangOption = {
  value: LangCode;
  label: string;
  countryCode?: string;
  documentLanguage: string;
  backendLanguage: string;
  dayjsLocale: string;
  numberFormat: { code: string };
};

export const allLangs: LangOption[] = supportedLocaleCodes.map(createLangOption);

function createLangOption(value: LangCode): LangOption {
  const contract = getLocaleContractEntry(value);
  const locale = new Intl.Locale(contract.documentLanguage);
  const maximized = locale.maximize();

  return {
    value,
    label: localeDisplayName(locale),
    countryCode: maximized.region,
    documentLanguage: contract.documentLanguage,
    backendLanguage: contract.backendLanguage,
    dayjsLocale: contract.dayjsLocale,
    numberFormat: { code: numberFormatLocale(maximized) },
  };
}

function localeDisplayName(locale: Intl.Locale): string {
  const displayNames = new Intl.DisplayNames([locale.baseName], { type: 'language' });
  return displayNames.of(locale.baseName) ?? locale.baseName;
}

function numberFormatLocale(locale: Intl.Locale): string {
  return locale.region ? `${locale.language}-${locale.region}` : locale.language;
}

// ----------------------------------------------------------------------

export const i18nResourceLoader = resourcesToBackend(loadTranslationResource);

export async function loadTranslationResource(
  lang: LangCode,
  namespace: string
): Promise<Record<string, unknown>> {
  getLocaleContractEntry(lang);
  assertI18nNamespace(namespace);

  try {
    return namespace === 'admin'
      ? await loadAdminTranslationResource(lang)
      : await loadNamespaceTranslationResource(lang, namespace);
  } catch (error) {
    throw new Error(`Locale resource is missing or unreadable: ${lang}/${namespace}`, {
      cause: error,
    });
  }
}

async function loadAdminTranslationResource(lang: LangCode): Promise<Record<string, unknown>> {
  const [base, navigation, dashboard, accessControl, profile, onlineSessions, notice, file] =
    await Promise.all([
      import(`./langs/${lang}/admin.json`),
      import(`./langs/${lang}/admin-navigation.json`),
      import(`./langs/${lang}/admin-dashboard.json`),
      import(`./langs/${lang}/admin-access-control.json`),
      import(`./langs/${lang}/admin-profile.json`),
      import(`./langs/${lang}/admin-online-sessions.json`),
      import(`./langs/${lang}/admin-notice.json`),
      import(`./langs/${lang}/admin-file.json`),
    ]);
  return mergeAdminResources(
    base.default,
    navigation.default,
    dashboard.default,
    accessControl.default,
    profile.default,
    onlineSessions.default,
    notice.default,
    file.default
  );
}

async function loadNamespaceTranslationResource(
  lang: LangCode,
  namespace: Exclude<I18nNamespace, 'admin'>
): Promise<Record<string, unknown>> {
  const resource = await import(`./langs/${lang}/${namespace}.json`);
  return resource.default;
}

function assertI18nNamespace(namespace: string): asserts namespace is I18nNamespace {
  if (!I18N_NAMESPACES.includes(namespace as I18nNamespace)) {
    throw new Error(`Unsupported i18n namespace: ${namespace}`);
  }
}

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
