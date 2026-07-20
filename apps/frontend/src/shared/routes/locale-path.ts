import type { LangCode } from 'src/shared/i18n/locales-config';

import { supportedLngs } from 'src/shared/i18n/locales-config';

const URL_SUFFIX_PATTERN = /([?#].*)$/;

export type LocaleRouteParams = Promise<{ locale: string }>;

export function isLangCode(value: string | undefined): value is LangCode {
  return Boolean(value && supportedLngs.includes(value as LangCode));
}

export function requireLangCode(value: string | undefined): LangCode {
  if (!isLangCode(value)) {
    throw new Error(`Unsupported route locale: ${value ?? '(missing)'}`);
  }

  return value;
}

export async function resolveRouteLocale(params: LocaleRouteParams): Promise<LangCode> {
  const { locale } = await params;
  return requireLangCode(locale);
}

export function localeFromPathname(pathname: string): LangCode | undefined {
  const locale = pathname.split('/').find(Boolean);
  return isLangCode(locale) ? locale : undefined;
}

export function stripLocalePrefix(path: string): string {
  const { pathname, suffix } = splitPath(path);
  const segments = pathname.split('/').filter(Boolean);
  const remainder = isLangCode(segments[0]) ? segments.slice(1) : segments;
  const trailingSlash = pathname.endsWith('/') && remainder.length > 0 ? '/' : '';
  return `/${remainder.join('/')}${trailingSlash}${suffix}`;
}

export function localizePath(locale: LangCode, path: string): string {
  if (isExternalPath(path)) return path;

  const localized = stripLocalePrefix(path);
  return localized === '/' ? `/${locale}/` : `/${locale}${localized}`;
}

function splitPath(path: string) {
  const [pathname, suffix = ''] = path.split(URL_SUFFIX_PATTERN);
  return { pathname: pathname || '/', suffix };
}

function isExternalPath(path: string) {
  return path.startsWith('#') || path.startsWith('?') || /^[a-z][a-z\d+.-]*:/i.test(path);
}
