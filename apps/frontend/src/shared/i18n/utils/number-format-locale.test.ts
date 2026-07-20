import { it, expect, describe } from 'vitest';

import { formatNumberLocale } from './number-format-locale';

describe('number format locale', () => {
  it('uses the locale prefix instead of the global i18next instance', () => {
    expect(formatNumberLocale('/en/dashboard/')).toEqual({ code: 'en-US', currency: 'USD' });
    expect(formatNumberLocale('/tw/dashboard/')).toEqual({ code: 'zh-TW', currency: 'TWD' });
    expect(formatNumberLocale('/cn/dashboard/')).toEqual({ code: 'zh-CN', currency: 'CNY' });
  });
});
