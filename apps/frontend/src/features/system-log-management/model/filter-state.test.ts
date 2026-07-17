import { it, expect, describe } from 'vitest';

import { createDefaultSystemLogFilters } from 'src/entities/system-log';

import { applySystemLogFilterDraft, resolveSystemLogActionFilter } from './filter-state';

describe('system log applied filters', () => {
  it('uses the submitted draft rather than mutating the active query while editing', () => {
    const currentQuery = { keyword: 'previous' };
    const draft = {
      ...createDefaultSystemLogFilters(new Date('2026-07-16T12:00:00.000Z')),
      keyword: ' next ',
    };

    expect(applySystemLogFilterDraft(currentQuery, draft)).toMatchObject({
      query: { keyword: 'next' },
      error: null,
      resetTable: true,
    });
    expect(currentQuery).toEqual({ keyword: 'previous' });
  });

  it('keeps the active query when a submitted time range is invalid', () => {
    const currentQuery = { keyword: 'previous' };
    const draft = {
      ...createDefaultSystemLogFilters(new Date('2026-07-16T12:00:00.000Z')),
      begin_time: '2026-07-16T12:00',
      end_time: '2026-07-16T11:00',
    };

    expect(applySystemLogFilterDraft(currentQuery, draft)).toEqual({
      query: currentQuery,
      error: 'invalid_range',
      resetTable: false,
    });
  });

  it('creates cleanup snapshots from the visible draft instead of a prior applied query', () => {
    const visibleDraft = {
      ...createDefaultSystemLogFilters(new Date('2026-07-16T12:00:00.000Z')),
      keyword: 'narrow scope',
    };

    expect(resolveSystemLogActionFilter(visibleDraft)).toMatchObject({
      kind: 'valid',
      query: { keyword: 'narrow scope' },
    });
  });
});
