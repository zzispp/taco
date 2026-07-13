import { it, expect, describe } from 'vitest';

import {
  toOnlineSessionQuery,
  ONLINE_SESSION_FILTER_ERROR,
  updateOnlineSessionFilterState,
  DEFAULT_ONLINE_SESSION_FILTERS,
} from './filters';

describe('online-session filter query conversion', () => {
  it('omits empty filters', () => {
    expect(toOnlineSessionQuery(DEFAULT_ONLINE_SESSION_FILTERS)).toEqual({
      ok: true,
      query: {},
    });
  });

  it('trims text and converts local login times to UTC RFC3339', () => {
    const result = toOnlineSessionQuery({
      ...DEFAULT_ONLINE_SESSION_FILTERS,
      ipaddr: ' 127.0.0.1 ',
      userName: ' admin ',
      begin_time: '2026-07-11T08:30',
      end_time: '2026-07-11T09:45',
    });

    expect(result).toEqual({
      ok: true,
      query: {
        ipaddr: '127.0.0.1',
        userName: 'admin',
        begin_time: new Date(2026, 6, 11, 8, 30).toISOString(),
        end_time: new Date(2026, 6, 11, 9, 45).toISOString(),
      },
    });
  });
});

describe('online-session filter validation', () => {
  it.each(['invalid', '2026-02-30T08:30', '2026-07-11T24:00'])(
    'rejects invalid local login time %s',
    (beginTime) => {
      expect(
        toOnlineSessionQuery({
          ...DEFAULT_ONLINE_SESSION_FILTERS,
          begin_time: beginTime,
        })
      ).toEqual({
        ok: false,
        error: ONLINE_SESSION_FILTER_ERROR.INVALID_DATE_TIME,
      });
    }
  );

  it('rejects a reversed login-time range', () => {
    expect(
      toOnlineSessionQuery({
        ...DEFAULT_ONLINE_SESSION_FILTERS,
        begin_time: '2026-07-11T09:45',
        end_time: '2026-07-11T08:30',
      })
    ).toEqual({
      ok: false,
      error: ONLINE_SESSION_FILTER_ERROR.INVALID_RANGE,
    });
  });

  it('retains the last valid query while exposing the invalid draft', () => {
    const previousQuery = { userName: 'admin' };
    const invalidDraft = {
      ...DEFAULT_ONLINE_SESSION_FILTERS,
      begin_time: '2026-07-11T09:45',
      end_time: '2026-07-11T08:30',
    };

    expect(updateOnlineSessionFilterState(previousQuery, invalidDraft)).toEqual({
      draft: invalidDraft,
      query: previousQuery,
      error: ONLINE_SESSION_FILTER_ERROR.INVALID_RANGE,
      resetTable: false,
    });
  });
});
