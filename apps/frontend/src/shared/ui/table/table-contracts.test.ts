import { createElement } from 'react';
import { it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import { TableNoData } from './table-no-data';
import { TableHeadCustom } from './table-head-custom';

describe('shared table contracts', () => {
  it('renders the caller-provided empty-state column span', () => {
    const markup = renderToStaticMarkup(
      createElement(TableNoData, { colSpan: 14, notFound: false })
    );

    expect(markup).toMatch(/colspan="14"/i);
  });

  it('renders the caller-provided localized select-all label', () => {
    const markup = renderToStaticMarkup(
      createElement(TableHeadCustom, {
        headCells: [],
        rowCount: 2,
        selectAllRowsLabel: '选择全部日志',
        sortStatusLabel: () => '按当前列升序排列',
        onSelectAllRows: () => undefined,
      })
    );

    expect(markup).toContain('aria-label="选择全部日志"');
  });

  it('renders the caller-provided localized sort status', () => {
    const markup = renderToStaticMarkup(
      createElement(TableHeadCustom, {
        headCells: [{ id: 'created_at', label: '创建时间' }],
        order: 'asc',
        orderBy: 'created_at',
        selectAllRowsLabel: '选择全部日志',
        sortStatusLabel: () => '按当前列升序排列',
        onSort: () => undefined,
      })
    );

    expect(markup).toContain('按当前列升序排列');
    expect(markup).not.toContain('sorted ascending');
  });
});
