import { it, expect, describe } from 'vitest';

import { preloadLocaleRuntime, requireLocaleSystemValue } from './locale-runtime';

describe('locale runtime', () => {
  it.each(['cn', 'en', 'tw'])(
    'loads the configured Dayjs and MUI resources for %s',
    async (lang) => {
      await expect(preloadLocaleRuntime(lang)).resolves.toBeUndefined();

      const { components } = requireLocaleSystemValue(lang);
      expect(Object.keys(components).length).toBeGreaterThan(0);
    }
  );
});
