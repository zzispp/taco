import { it, expect, describe } from 'vitest';

import {
  localizePath,
  requireLangCode,
  stripLocalePrefix,
  localeFromPathname,
} from './locale-path';

describe('locale route paths', () => {
  it('reads only supported leading locale segments', () => {
    expect(localeFromPathname('/en/auth/sign-in/')).toBe('en');
    expect(localeFromPathname('/fr/auth/sign-in/')).toBeUndefined();
    expect(localeFromPathname('/dashboard/')).toBeUndefined();
  });

  it('prefixes internal paths while preserving query strings and fragments', () => {
    expect(localizePath('tw', '/dashboard/?tab=profile#security')).toBe(
      '/tw/dashboard/?tab=profile#security'
    );
    expect(localizePath('en', '/cn/auth/sign-in/')).toBe('/en/auth/sign-in/');
    expect(localizePath('cn', '/')).toBe('/cn/');
  });

  it('leaves external and document-local paths unchanged', () => {
    expect(localizePath('en', 'https://example.test/docs')).toBe('https://example.test/docs');
    expect(localizePath('en', '#help')).toBe('#help');
  });

  it('strips only a supported locale prefix and rejects missing route locales', () => {
    expect(stripLocalePrefix('/tw/dashboard/')).toBe('/dashboard/');
    expect(stripLocalePrefix('/dashboard/')).toBe('/dashboard/');
    expect(() => requireLangCode('fr')).toThrow('Unsupported route locale: fr');
  });
});
