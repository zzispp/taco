import type { useTranslate } from 'src/shared/i18n/use-locales';

import { it, expect, describe } from 'vitest';

import { configFields } from './config-fields';

const translate = ((key: string) => key) as ReturnType<typeof useTranslate>['t'];

describe('config fields', () => {
  it('uses a multiline input for the config value', () => {
    const field = configFields(translate).find((item) => item.key === 'config_value');

    expect(field?.type).toBe('textarea');
  });
});
