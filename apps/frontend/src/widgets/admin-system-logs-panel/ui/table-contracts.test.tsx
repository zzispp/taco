import type { ReactElement } from 'react';
import type { SystemLogController } from 'src/features/system-log-management';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import Table from '@mui/material/Table';
import TableBody from '@mui/material/TableBody';
import { ThemeProvider } from '@mui/material/styles';

import { createTheme } from 'src/shared/theme';
import { TableNoData } from 'src/shared/ui/table';

import { withSelectionHead, ManagementTableHead } from 'src/widgets/admin-common';

import { SystemLogRow } from './system-log-row';
import { SYSTEM_LOG_HEAD, systemLogTableHeads } from './table-section';

const TEST_THEME = createTheme();

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({
    t: (key: string) => (key === 'levels.info' ? 'Info level' : key),
  }),
}));

describe('system log table contracts', () => {
  it('renders 6 columns without selection and 7 with selection', () => {
    const base = renderToStaticMarkup(
      createElement(TableNoData, { colSpan: SYSTEM_LOG_HEAD.length, notFound: false })
    );
    const selectable = renderToStaticMarkup(
      createElement(TableNoData, {
        colSpan: withSelectionHead([...SYSTEM_LOG_HEAD]).length,
        notFound: false,
      })
    );

    expect(base).toMatch(/colspan="6"/i);
    expect(selectable).toMatch(/colspan="7"/i);
  });

  it('renders the log identifier before its level', () => {
    const controller = {
      resources: { canRemove: false, canQuery: false },
      pending: new Set<string>(),
      state: { table: { selected: [], onSelectRow: vi.fn() } },
      actions: {},
    } as unknown as SystemLogController;
    const markup = renderWithTheme(
      createElement(SystemLogRow, {
        controller,
        log: {
          log_id: 'system-log-identifier',
          occurred_at: '2026-07-17T00:00:00.000Z',
          level: 'info',
          target: 'test::system_log',
          message: 'System log message',
        },
      })
    );

    expect(markup.indexOf('system-log-identifier')).toBeLessThan(markup.indexOf('Info level'));
    expect(markup.indexOf('Info level')).toBeLessThan(markup.indexOf('test::system_log'));
  });

  it('aligns the selectable header with every row cell', () => {
    const controller = selectableController();
    const heads = systemLogTableHeads((key) => key, true);
    const markup = renderWithTheme(
      <Table>
        <ManagementTableHead
          head={heads.header}
          rowCount={1}
          numSelected={0}
          onSelectAllRows={vi.fn()}
        />
        <TableBody>
          <SystemLogRow controller={controller} log={testLog()} />
        </TableBody>
      </Table>
    );

    expect(tableCellCount(markup, 'th')).toBe(heads.body.length);
    expect(tableCellCount(markup, 'td')).toBe(heads.body.length);
  });
});

function renderWithTheme(element: ReactElement) {
  return renderToStaticMarkup(createElement(ThemeProvider, { theme: TEST_THEME }, element));
}

function selectableController() {
  return {
    resources: { canRemove: true, canQuery: false },
    pending: new Set<string>(),
    state: { table: { selected: [], onSelectRow: vi.fn() } },
    actions: {},
  } as unknown as SystemLogController;
}

function testLog() {
  return {
    log_id: 'system-log-identifier',
    occurred_at: '2026-07-17T00:00:00.000Z',
    level: 'info' as const,
    target: 'test::system_log',
    message: 'System log message',
  };
}

function tableCellCount(markup: string, tag: 'td' | 'th') {
  return (markup.match(new RegExp(`<${tag}\\b`, 'g')) ?? []).length;
}
