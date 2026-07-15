import type { useTranslate } from 'src/shared/i18n/use-locales';

import { it, expect, describe } from 'vitest';

import { paths } from 'src/shared/routes/paths';

import { translateNavData } from './nav-translation';

describe('overview navigation translation', () => {
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
});

describe('system monitoring navigation translation', () => {
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

describe('audit navigation translation', () => {
  it('translates the nested audit log menu by stable menu code', () => {
    const translate = ((key: string) => key) as ReturnType<typeof useTranslate>['t'];
    const translated = translateNavData(
      [
        {
          code: '3',
          subheader: '系统监控',
          items: [
            {
              code: '111',
              title: '日志管理',
              path: paths.dashboard.monitorLogs.root,
              icon: 'icon.logs',
              deepMatch: true,
              children: [
                {
                  code: '112',
                  title: '操作日志',
                  path: paths.dashboard.monitorLogs.operationLogs,
                  deepMatch: false,
                },
                {
                  code: '113',
                  title: '登录日志',
                  path: paths.dashboard.monitorLogs.loginLogs,
                  deepMatch: false,
                },
              ],
            },
          ],
        },
      ],
      translate
    );

    expect(translated[0].items[0].title).toBe('nav.logManagement');
    expect(translated[0].items[0].deepMatch).toBe(true);
    expect(translated[0].items[0].children?.map((child) => child.title)).toEqual([
      'nav.operationLogs',
      'nav.loginLogs',
    ]);
  });
});

describe('custom navigation labels', () => {
  it('keeps custom section and item labels verbatim even when they match built-in titles', () => {
    const translate = ((key: string) => key) as ReturnType<typeof useTranslate>['t'];
    const translated = translateNavData(
      [
        {
          code: 'custom-section',
          subheader: '系统管理',
          items: [
            {
              code: 'custom-item',
              title: '用户管理',
              path: '/dashboard/custom/users',
              caption: 'Custom keyboard shortcuts.',
              deepMatch: false,
            },
          ],
        },
      ],
      translate
    );

    expect(translated[0].subheader).toBe('系统管理');
    expect(translated[0].items[0].title).toBe('用户管理');
    expect(translated[0].items[0].caption).toBe('Custom keyboard shortcuts.');
  });
});
