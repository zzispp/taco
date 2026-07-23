import { it, expect, describe } from 'vitest';

import { parseLocaleContract } from './locale-contract';

describe('locale contract', () => {
  it('accepts a locale added through the contract without a source-code whitelist', () => {
    const contract = parseLocaleContract({
      defaultLocale: 'de',
      locales: [
        {
          code: 'de',
          documentLanguage: 'de',
          backendLanguage: 'de',
          dayjsLocale: 'de',
          muiLocale: 'deDE',
        },
      ],
    });

    expect(contract.defaultLocale).toBe('de');
    expect(contract.locales.map((locale) => locale.code)).toEqual(['de']);
  });

  it('keeps runtime module mappings explicit for regional document languages', () => {
    const contract = parseLocaleContract({
      defaultLocale: 'en',
      locales: [
        {
          code: 'en',
          documentLanguage: 'en-US',
          backendLanguage: 'en',
          dayjsLocale: 'en',
          muiLocale: 'enUS',
        },
      ],
    });

    expect(contract.locales[0]).toMatchObject({
      documentLanguage: 'en-US',
      dayjsLocale: 'en',
      muiLocale: 'enUS',
    });
  });

  it('rejects duplicate codes and a default locale outside the contract', () => {
    expect(() =>
      parseLocaleContract({
        defaultLocale: 'cn',
        locales: [
          {
            code: 'cn',
            documentLanguage: 'zh-CN',
            backendLanguage: 'zh-CN',
            dayjsLocale: 'zh-cn',
            muiLocale: 'zhCN',
          },
          {
            code: 'cn',
            documentLanguage: 'zh-CN',
            backendLanguage: 'zh-CN',
            dayjsLocale: 'zh-cn',
            muiLocale: 'zhCN',
          },
        ],
      })
    ).toThrow('unique locale codes');
    expect(() => parseLocaleContract({ defaultLocale: 'en', locales: [] })).toThrow(
      'unique locale codes'
    );
  });
});
