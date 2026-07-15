import type { TranslateFn } from 'src/shared/i18n';

import { isValidElement } from 'react';
import { it, expect, describe } from 'vitest';

import { paths } from 'src/shared/routes/paths';

import { NAV_ICONS, NAV_ICON_OPTIONS, translatedMenuItem } from './nav-metadata';

const AUDIT_MENU_ICON_TOKENS = ['icon.logs', 'icon.operation-log', 'icon.login-log'] as const;

describe('audit menu icon metadata', () => {
  it.each(AUDIT_MENU_ICON_TOKENS)('registers %s as a selectable rendered icon', (token) => {
    expect(NAV_ICON_OPTIONS).toContain(token);
    expect(isValidElement(NAV_ICONS[token])).toBe(true);
  });
});

describe('audit menu translation metadata', () => {
  it.each([
    [paths.dashboard.monitorLogs.root, 'nav.logManagement'],
    [paths.dashboard.monitorLogs.operationLogs, 'nav.operationLogs'],
    [paths.dashboard.monitorLogs.loginLogs, 'nav.loginLogs'],
  ])('translates %s from its path', (path, key) => {
    expect(translatedMenuItem(menu({ path }), translate)).toBe(key);
  });

  it('keeps custom menu names even when their permission resembles a built-in menu', () => {
    const custom = menu({ path: '/dashboard/custom/audit', perms: 'system:operlog:list' });
    custom.menu_id = 'custom-audit';

    expect(translatedMenuItem(custom, translate)).toBe('日志管理');
  });
});

const translate = ((key: string) => key) as TranslateFn;

function menu(overrides: Partial<{ path: string; perms: string | null }> = {}) {
  return {
    menu_id: 'audit',
    menu_name: '日志管理',
    parent_id: '3',
    order_num: 1,
    path: overrides.path ?? paths.dashboard.monitorLogs.root,
    component: null,
    query: '',
    route_name: 'AuditLogs',
    is_frame: false,
    is_cache: false,
    menu_type: 'C',
    visible: '0',
    status: '0',
    perms: overrides.perms ?? null,
    icon: 'icon.logs',
    remark: null,
  };
}
