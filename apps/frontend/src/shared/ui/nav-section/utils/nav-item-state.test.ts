import type { NavItemDataProps } from '../types';

import { it, expect, describe } from 'vitest';

import { isNavItemActive, isNavBranchActive } from './nav-item-state';

const logDirectory: NavItemDataProps = {
  title: '日志管理',
  path: '/dashboard/monitor/logs',
  deepMatch: false,
  children: [
    {
      title: '操作日志',
      path: '/dashboard/monitor/logs/operation-logs',
      deepMatch: false,
    },
    {
      title: '登录日志',
      path: '/dashboard/monitor/logs/login-logs',
      deepMatch: false,
    },
  ],
};

describe('navigation branch state', () => {
  it('keeps an exact-match directory branch active while switching between its children', () => {
    const routes = ['/dashboard/monitor/logs/operation-logs', '/dashboard/monitor/logs/login-logs'];

    expect(routes.map((route) => isNavItemActive(route, logDirectory))).toEqual([false, false]);
    expect(routes.map((route) => isNavBranchActive(route, logDirectory))).toEqual([true, true]);
  });

  it('does not activate the directory for an unrelated route with a similar prefix', () => {
    expect(isNavBranchActive('/dashboard/monitor/logs-archive', logDirectory)).toBe(false);
  });

  it('compares localized route paths against locale-neutral navigation items', () => {
    expect(isNavBranchActive('/en/dashboard/monitor/logs/login-logs/', logDirectory)).toBe(true);
  });
});
