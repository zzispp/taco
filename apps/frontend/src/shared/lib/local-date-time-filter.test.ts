import { it, expect, describe } from 'vitest';

import {
  LOCAL_DATE_TIME_FILTER_ERROR,
  createLocalDateTimeFilterState,
  updateLocalDateTimeFilterState,
  transitionLocalDateTimeFilterState,
} from './local-date-time-filter';

const DEFAULT_FILTERS = Object.freeze({
  name: '',
  begin_time: '',
  end_time: '',
});

describe('local date-time filter state', () => {
  it('creates an independent UTC query from a local draft', () => {
    const draft = {
      name: 'admin',
      begin_time: '2026-07-11T08:30',
      end_time: '2026-07-11T09:45',
    };

    expect(createLocalDateTimeFilterState(draft)).toEqual({
      draft,
      query: {
        name: 'admin',
        begin_time: new Date(2026, 6, 11, 8, 30).toISOString(),
        end_time: new Date(2026, 6, 11, 9, 45).toISOString(),
      },
      error: null,
    });
  });

  it('keeps empty date filters empty', () => {
    expect(createLocalDateTimeFilterState(DEFAULT_FILTERS)).toEqual({
      draft: DEFAULT_FILTERS,
      query: DEFAULT_FILTERS,
      error: null,
    });
  });

  it.each(['invalid', '2026-02-30T08:30', '2026-07-11T24:00'])(
    'retains the last valid query for invalid draft %s',
    (beginTime) => {
      const previousQuery = { name: 'admin', begin_time: '', end_time: '' };
      const draft = { ...DEFAULT_FILTERS, begin_time: beginTime };

      const transition = updateLocalDateTimeFilterState(previousQuery, draft);

      expect(transition).toEqual({
        draft,
        query: previousQuery,
        error: LOCAL_DATE_TIME_FILTER_ERROR.INVALID_DATE_TIME,
        resetTable: false,
      });
      expect(transition.query).toBe(previousQuery);
    }
  );

  it('retains the last valid query for a reversed range', () => {
    const previousQuery = { name: 'admin', begin_time: '', end_time: '' };
    const draft = {
      ...DEFAULT_FILTERS,
      begin_time: '2026-07-11T09:45',
      end_time: '2026-07-11T08:30',
    };

    const transition = updateLocalDateTimeFilterState(previousQuery, draft);

    expect(transition).toEqual({
      draft,
      query: previousQuery,
      error: LOCAL_DATE_TIME_FILTER_ERROR.INVALID_RANGE,
      resetTable: false,
    });
    expect(transition.query).toBe(previousQuery);
  });

  it('returns a UTC query and requests a page reset for a valid change', () => {
    const draft = { ...DEFAULT_FILTERS, begin_time: '2026-07-11T08:30' };

    expect(updateLocalDateTimeFilterState(DEFAULT_FILTERS, draft)).toEqual({
      draft,
      query: {
        ...DEFAULT_FILTERS,
        begin_time: new Date(2026, 6, 11, 8, 30).toISOString(),
      },
      error: null,
      resetTable: true,
    });
  });

  it('serializes a valid update before an invalid draft in the same batch', () => {
    const validDraft = { ...DEFAULT_FILTERS, begin_time: '2026-07-11T08:30' };
    const valid = transitionLocalDateTimeFilterState(
      createLocalDateTimeFilterState(DEFAULT_FILTERS),
      validDraft
    );
    const invalidDraft = { ...validDraft, end_time: 'invalid' };

    const invalid = transitionLocalDateTimeFilterState(valid.state, invalidDraft);

    expect(valid.resetTable).toBe(true);
    expect(invalid).toEqual({
      state: {
        draft: invalidDraft,
        query: valid.state.query,
        error: LOCAL_DATE_TIME_FILTER_ERROR.INVALID_DATE_TIME,
      },
      resetTable: false,
    });
    expect(invalid.state.query).toBe(valid.state.query);
  });
});
