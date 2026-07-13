import type { useTranslate } from 'src/shared/i18n/use-locales';

import { it, expect, describe } from 'vitest';

import { translateNavData } from './nav-translation';

describe('dashboard navigation translation', () => {
  it('translates the database-backed overview directory and dashboard item', () => {
    const translate = ((key: string) => key) as ReturnType<typeof useTranslate>['t'];
    const translated = translateNavData(
      [
        {
          code: '4',
          subheader: '概览',
          items: [
            {
              code: '2',
              title: '仪表盘',
              path: '/dashboard',
              icon: 'icon.dashboard',
              deepMatch: false,
            },
          ],
        },
      ],
      translate
    );

    expect(translated[0].subheader).toBe('nav.overview');
    expect(translated[0].items[0].title).toBe('nav.dashboard');
  });

  it('translates the system monitoring section and its moved items', () => {
    const translate = ((key: string) => key) as ReturnType<typeof useTranslate>['t'];
    const translated = translateNavData(
      [
        {
          code: '3',
          subheader: '系统监控',
          items: [
            {
              code: '107',
              title: '在线用户',
              path: '/dashboard/admin/online',
              icon: 'icon.online',
              deepMatch: false,
            },
          ],
        },
      ],
      translate
    );

    expect(translated[0].subheader).toBe('nav.systemMonitor');
    expect(translated[0].items[0].title).toBe('nav.online');
  });
});
