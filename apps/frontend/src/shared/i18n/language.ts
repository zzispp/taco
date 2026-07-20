import type { LangCode } from './locales-config';

const DEFAULT_QUALITY = 1;
const MIN_QUALITY = 0;
const MAX_QUALITY = 1;

const BACKEND_LOCALE_BY_LANGUAGE: Readonly<Record<LangCode, string>> = Object.freeze({
  cn: 'zh-CN',
  en: 'en',
  tw: 'zh-TW',
});

const DOCUMENT_LANGUAGE_BY_LANGUAGE: Readonly<Record<LangCode, string>> = Object.freeze({
  cn: 'zh-CN',
  en: 'en',
  tw: 'zh-TW',
});

type LanguagePreference = Readonly<{
  language: LangCode;
  quality: number;
  order: number;
}>;

export function normalizeLanguage(value?: string | null): LangCode | undefined {
  if (!value) return undefined;

  const language = value.trim().toLowerCase().replaceAll('_', '-');
  if (isTraditionalChinese(language)) return 'tw';
  if (isSimplifiedChinese(language)) return 'cn';
  if (language === 'en' || language.startsWith('en-')) return 'en';
  return undefined;
}

export function toBackendAcceptLanguage(value?: string | null): string | undefined {
  const language = normalizeLanguage(value);
  return language ? BACKEND_LOCALE_BY_LANGUAGE[language] : undefined;
}

export function toDocumentLanguage(value?: string | null): string | undefined {
  const language = normalizeLanguage(value);
  return language ? DOCUMENT_LANGUAGE_BY_LANGUAGE[language] : undefined;
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

function isTraditionalChinese(language: string): boolean {
  return (
    language === 'tw' ||
    language.startsWith('zh-tw') ||
    language.startsWith('zh-hk') ||
    language.startsWith('zh-mo') ||
    language.startsWith('zh-hant')
  );
}

function isSimplifiedChinese(language: string): boolean {
  return (
    language === 'cn' ||
    language === 'zh' ||
    language.startsWith('zh-cn') ||
    language.startsWith('zh-hans')
  );
}
