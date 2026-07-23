import { it, expect, describe } from 'vitest';

import { formatNumberLocale } from './number-format-locale';

describe('number format locale', () => {
  it('uses the locale prefix instead of the global i18next instance', () => {
    expect(formatNumberLocale('/en/dashboard/')).toEqual({ code: 'en-US' });
    expect(formatNumberLocale('/tw/dashboard/')).toEqual({ code: 'zh-TW' });
    expect(formatNumberLocale('/cn/dashboard/')).toEqual({ code: 'zh-CN' });
  });
});
