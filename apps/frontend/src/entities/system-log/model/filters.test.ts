import { it, expect, describe } from 'vitest';

import {
  toSystemLogQuery,
  hasRequiredSystemLogRange,
  createDefaultSystemLogFilters,
} from './filters';

describe('system log filters', () => {
  it('initializes a full-precision 24-hour range for cursor-stable querying', () => {
    const now = new Date('2026-07-16T12:00:59.789Z');

    const filters = createDefaultSystemLogFilters(now);
    const query = toSystemLogQuery(filters);

    expect(query).toMatchObject({
      ok: true,
      query: {
        begin_time: new Date(now.getTime() - 24 * 60 * 60 * 1000).toISOString(),
        end_time: now.toISOString(),
      },
    });
  });

  it('normalizes search text and multi-level selections for the backend contract', () => {
    const query = toSystemLogQuery({
      keyword: ' request ',
      target: ' taco ',
      levels: ['error', 'info', 'error'],
      begin_time: '2026-07-15T08:00',
      end_time: '2026-07-16T08:00',
    });

    expect(query).toMatchObject({
      ok: true,
      query: {
        keyword: 'request',
        target: 'taco',
        levels: 'error,info',
      },
    });
    expect(query.ok && hasRequiredSystemLogRange(query.query)).toBe(true);
  });

  it('retains date validation failures instead of producing an invalid query', () => {
    const result = toSystemLogQuery({
      keyword: '',
      target: '',
      levels: [],
      begin_time: 'invalid',
      end_time: '2026-07-16T08:00',
    });

    expect(result).toEqual({ ok: false, error: 'invalid_date_time' });
  });
});
