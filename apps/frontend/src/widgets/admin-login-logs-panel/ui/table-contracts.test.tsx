import type { ReactElement } from 'react';
import type { LoginLogController } from 'src/features/audit-log-management';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import { ThemeProvider } from '@mui/material/styles';

import { createTheme } from 'src/shared/theme';
import { TableNoData } from 'src/shared/ui/table';

import { withSelectionHead } from 'src/widgets/admin-common';

import { LoginLogRow } from './login-log-row';
import { LOGIN_LOG_HEAD } from './table-section';

const TEST_THEME = createTheme();

function renderWithTheme(element: ReactElement) {
  return renderToStaticMarkup(createElement(ThemeProvider, { theme: TEST_THEME }, element));
}

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({
    t: (key: string, options?: { id?: string }) =>
      key === 'table.selectRow' ? `Select login ${options?.id}` : key,
  }),
}));

describe('login log table contracts', () => {
  it('keeps the login-time column immediately before row actions', () => {
    const columns = LOGIN_LOG_HEAD.map(({ id }) => id);

    expect(columns).toContain('login_time');
    expect(columns.indexOf('login_time')).toBe(columns.indexOf('actions') - 1);
  });

  it('renders 11 columns without selection and 12 with selection', () => {
    const base = renderToStaticMarkup(
      createElement(TableNoData, { colSpan: LOGIN_LOG_HEAD.length, notFound: false })
    );
    const selectable = renderToStaticMarkup(
      createElement(TableNoData, {
        colSpan: withSelectionHead([...LOGIN_LOG_HEAD]).length,
        notFound: false,
      })
    );

    expect(base).toMatch(/colspan="11"/i);
    expect(selectable).toMatch(/colspan="12"/i);
  });

  it('renders the localized row selection label with the login id', () => {
    const controller = {
      state: { table: { selected: [], onSelectRow: vi.fn() }, setDeleteTarget: vi.fn() },
      resources: { canRemove: true, canUnlock: false },
      pending: new Set<string>(),
    } as unknown as LoginLogController;
    const markup = renderWithTheme(
      createElement(LoginLogRow, {
        controller,
        log: {
          info_id: 'login-1',
          user_name: 'alice',
          ipaddr: '198.51.100.8',
          login_location: 'Provider Text',
          browser: 'Chrome',
          os: 'macOS',
          status: 0,
          msg: 'Login succeeded',
          event_type: 'login_success',
          login_time: '2026-07-13T00:00:00Z',
        },
      })
    );

    expect(markup).toContain('aria-label="Select login login-1"');
  });
});
