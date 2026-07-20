import { it, expect, describe } from 'vitest';

import {
  normalizeLanguage,
  toDocumentLanguage,
  updateDocumentLanguage,
  resolveAcceptedLanguage,
  toBackendAcceptLanguage,
} from './language';

describe('language normalization', () => {
  it.each([
    ['cn', 'cn'],
    ['zh', 'cn'],
    ['zh_Hans_CN', 'cn'],
    ['zh-CN', 'cn'],
    ['tw', 'tw'],
    ['zh-Hant-HK', 'tw'],
    ['zh-HK', 'tw'],
    ['zh-MO', 'tw'],
    ['en', 'en'],
    ['en-GB', 'en'],
  ] as const)('normalizes %s to %s', (input, expected) => {
    expect(normalizeLanguage(input)).toBe(expected);
  });

  it('maps frontend language codes to backend wire locales', () => {
    expect(toBackendAcceptLanguage('cn')).toBe('zh-CN');
    expect(toBackendAcceptLanguage('en-US')).toBe('en');
    expect(toBackendAcceptLanguage('tw')).toBe('zh-TW');
    expect(toBackendAcceptLanguage('fr')).toBeUndefined();
  });

  it('maps supported language codes to document language tags', () => {
    expect(toDocumentLanguage('cn')).toBe('zh-CN');
    expect(toDocumentLanguage('en-US')).toBe('en');
    expect(toDocumentLanguage('zh-Hant')).toBe('zh-TW');
    expect(toDocumentLanguage('fr')).toBeUndefined();
  });

  it('updates the document language only for supported locales', () => {
    const documentElement = { lang: 'zh-CN' };

    updateDocumentLanguage(documentElement, 'en-US');
    expect(documentElement.lang).toBe('en');

    updateDocumentLanguage(documentElement, 'fr-FR');
    expect(documentElement.lang).toBe('en');
  });
});

describe('Accept-Language negotiation', () => {
  it('selects the supported locale with the highest quality', () => {
    expect(resolveAcceptedLanguage('en;q=0.2, zh-CN;q=0.8, zh-TW;q=0.9')).toBe('tw');
  });

  it('preserves header order for equal quality values', () => {
    expect(resolveAcceptedLanguage('en;q=0.8, zh-TW;q=0.8')).toBe('en');
  });

  it('excludes zero-quality and malformed preferences', () => {
    expect(resolveAcceptedLanguage('zh-TW;q=0, en;q=invalid, zh-CN;q=0.7')).toBe('cn');
  });

  it('returns undefined when no supported preference is acceptable', () => {
    expect(resolveAcceptedLanguage('fr-FR, *;q=0.8, en;q=0')).toBeUndefined();
  });
});
