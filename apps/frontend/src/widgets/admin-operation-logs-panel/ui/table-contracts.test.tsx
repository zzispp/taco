import type { ReactElement } from 'react';
import type { OperationLogController } from 'src/features/audit-log-management';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import { ThemeProvider } from '@mui/material/styles';

import { createTheme } from 'src/shared/theme';
import { TableNoData } from 'src/shared/ui/table';

import { withSelectionHead } from 'src/widgets/admin-common';

import { OPERATION_LOG_HEAD } from './table-section';
import { OperationLogRow } from './operation-log-row';

const TEST_THEME = createTheme();

function renderWithTheme(element: ReactElement) {
  return renderToStaticMarkup(createElement(ThemeProvider, { theme: TEST_THEME }, element));
}

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({
    t: (key: string, options?: { id?: string }) =>
      key === 'table.selectRow' ? `Select operation ${options?.id}` : key,
  }),
}));

describe('operation log table contracts', () => {
  it('renders 13 columns without selection and 14 with selection', () => {
    const base = renderToStaticMarkup(
      createElement(TableNoData, { colSpan: OPERATION_LOG_HEAD.length, notFound: false })
    );
    const selectable = renderToStaticMarkup(
      createElement(TableNoData, {
        colSpan: withSelectionHead([...OPERATION_LOG_HEAD]).length,
        notFound: false,
      })
    );

    expect(base).toMatch(/colspan="13"/i);
    expect(selectable).toMatch(/colspan="14"/i);
  });

  it('renders the localized row selection label with the operation id', () => {
    const controller = {
      state: { table: { selected: [], onSelectRow: vi.fn() }, setDeleteTarget: vi.fn() },
      resources: { canRemove: true, canQuery: false },
      pending: new Set<string>(),
      actions: {},
    } as unknown as OperationLogController;
    const markup = renderWithTheme(
      createElement(OperationLogRow, {
        controller,
        log: {
          oper_id: 'operation-1',
          title: 'User management',
          business_type: 2,
          method: 'user::replace',
          request_method: 'PUT',
          operator_type: 1,
          oper_name: 'admin',
          dept_name: 'Platform',
          oper_url: '/api/system/users/user-1',
          oper_ip: '198.51.100.8',
          oper_location: 'Provider Text',
          status: 0,
          oper_time: '2026-07-13T00:00:00Z',
          cost_time: 5,
        },
      })
    );

    expect(markup).toContain('aria-label="Select operation operation-1"');
  });
});
