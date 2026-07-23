import type { LangCode } from './locales-config';

import { getCurrentLang } from './locales-config';
import { supportedLocaleCodes, getLocaleContractEntry } from './locale-contract';

const DEFAULT_QUALITY = 1;
const MIN_QUALITY = 0;
const MAX_QUALITY = 1;

type LanguagePreference = Readonly<{
  language: LangCode;
  quality: number;
  order: number;
}>;

export function normalizeLanguage(value?: string | null): LangCode | undefined {
  if (!value) return undefined;

  const language = value.trim().replaceAll('_', '-');
  const code = supportedLocaleCodes.find((entry) => entry.toLowerCase() === language.toLowerCase());
  if (code) return code;

  const requestedLocale = parseLocale(language);
  return requestedLocale ? matchingContractLocale(requestedLocale) : undefined;
}

export function toBackendAcceptLanguage(value?: string | null): string | undefined {
  const language = normalizeLanguage(value);
  return language ? languageOption(language).backendLanguage : undefined;
}

export function toDocumentLanguage(value?: string | null): string | undefined {
  const language = normalizeLanguage(value);
  return language ? languageOption(language).documentLanguage : undefined;
}

export function updateDocumentLanguage(element: { lang: string }, value?: string | null) {
  const language = toDocumentLanguage(value);
  if (language) element.lang = language;
}

export function resolveAcceptedLanguage(header?: string | null): LangCode | undefined {
  if (!header) return undefined;

  const preferences = header
    .split(',')
    .map(parsePreference)
    .filter((preference): preference is LanguagePreference => Boolean(preference))
    .filter(({ quality }) => quality > MIN_QUALITY)
    .sort((left, right) => right.quality - left.quality || left.order - right.order);

  return preferences[0]?.language;
}

function parsePreference(value: string, order: number): LanguagePreference | undefined {
  const [tag, ...parameters] = value.split(';');
  const language = normalizeLanguage(tag);
  const quality = parseQuality(parameters);
  if (!language || quality === undefined) return undefined;
  return { language, quality, order };
}

function parseQuality(parameters: string[]): number | undefined {
  const qualityParameter = parameters.find((parameter) => /^\s*q\s*=/i.test(parameter));
  if (!qualityParameter) return DEFAULT_QUALITY;

  const match = /^\s*q\s*=\s*(\d(?:\.\d{0,3})?)\s*$/i.exec(qualityParameter);
  if (!match) return undefined;
  const quality = Number(match[1]);
  return quality >= MIN_QUALITY && quality <= MAX_QUALITY ? quality : undefined;
}

function matchingContractLocale(requestedLocale: Intl.Locale): LangCode | undefined {
  const locales = supportedLocaleCodes.map((code) => ({
    code,
    locale: new Intl.Locale(getLocaleContractEntry(code).documentLanguage),
  }));
  const exact = locales.find(({ locale }) => locale.baseName === requestedLocale.baseName);
  if (exact) return exact.code;

  const requestedMaximized = requestedLocale.maximize();
  const maximized = locales.map(({ code, locale }) => ({ code, locale: locale.maximize() }));
  const exactMaximized = maximized.find(
    ({ locale }) => locale.baseName === requestedMaximized.baseName
  );
  if (exactMaximized) return exactMaximized.code;

  const sameScript = maximized.find(
    ({ locale }) =>
      locale.language === requestedMaximized.language && locale.script === requestedMaximized.script
  );
  return (
    sameScript?.code ??
    maximized.find(({ locale }) => locale.language === requestedMaximized.language)?.code
  );
}

function parseLocale(value: string): Intl.Locale | undefined {
  try {
    return new Intl.Locale(value);
  } catch {
    return undefined;
  }
}

function languageOption(language: LangCode) {
  const option = getCurrentLang(language);
  if (option.value !== language) {
    throw new Error(`Locale contract is missing language: ${language}`);
  }
  return option;
}
