import localeContract from '../../../../../locale-contract.json';

export type LocaleCode = string;

export type LocaleEntry = Readonly<{
  code: LocaleCode;
  documentLanguage: string;
  backendLanguage: string;
  dayjsLocale: string;
  muiLocale: string;
}>;

type LocaleContract = Readonly<{
  defaultLocale: LocaleCode;
  locales: readonly LocaleEntry[];
}>;

const contract = parseLocaleContract(localeContract);

export const supportedLocaleCodes = contract.locales.map(({ code }) => code);
export const defaultLocaleCode = contract.defaultLocale;
export const defaultLocaleHomePath = `/${defaultLocaleCode}/`;
export const defaultDocumentLanguage = getLocaleContractEntry(defaultLocaleCode).documentLanguage;

export function getLocaleContractEntry(code: LocaleCode): LocaleEntry {
  const locale = contract.locales.find((entry) => entry.code === code);
  if (locale) return locale;
  throw new Error(`Locale contract is missing language: ${code}`);
}

export function parseLocaleContract(value: unknown): LocaleContract {
  if (
    !isRecord(value) ||
    typeof value.defaultLocale !== 'string' ||
    !Array.isArray(value.locales)
  ) {
    throw new Error('Locale contract must contain defaultLocale and locales');
  }
  const locales = value.locales.map(parseLocaleEntry);
  ensureUniqueLocaleCodes(locales);
  if (!locales.some((locale) => locale.code === value.defaultLocale)) {
    throw new Error(`Locale contract default language is missing: ${value.defaultLocale}`);
  }
  return { defaultLocale: value.defaultLocale, locales };
}

function parseLocaleEntry(value: unknown): LocaleEntry {
  if (
    !isRecord(value) ||
    !isLocaleText(value.code) ||
    !isLocaleText(value.documentLanguage) ||
    !isLocaleText(value.backendLanguage) ||
    !isLocaleText(value.dayjsLocale) ||
    !isLocaleText(value.muiLocale)
  ) {
    throw new Error('Locale contract contains an invalid locale entry');
  }
  return {
    code: value.code,
    documentLanguage: value.documentLanguage,
    backendLanguage: value.backendLanguage,
    dayjsLocale: value.dayjsLocale,
    muiLocale: value.muiLocale,
  };
}

function ensureUniqueLocaleCodes(locales: readonly LocaleEntry[]) {
  const codes = new Set(locales.map((locale) => locale.code));
  if (!locales.length || codes.size !== locales.length) {
    throw new Error('Locale contract must contain unique locale codes');
  }
}

function isLocaleText(value: unknown): value is string {
  return typeof value === 'string' && Boolean(value.trim());
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
