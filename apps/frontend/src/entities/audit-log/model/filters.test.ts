import { it, expect, describe } from 'vitest';

import {
  toLoginLogQuery,
  AUDIT_FILTER_ERROR,
  toOperationLogQuery,
  DEFAULT_LOGIN_LOG_FILTERS,
  DEFAULT_OPERATION_LOG_FILTERS,
  updateOperationLogFilterState,
} from './filters';

describe('operation log query conversion', () => {
  it('omits empty filters and trims text values', () => {
    expect(
      toOperationLogQuery({
        ...DEFAULT_OPERATION_LOG_FILTERS,
        title: ' 用户管理 ',
        oper_name: ' admin ',
        oper_ip: ' 10.0.0.5 ',
        business_type: 1,
        status: 0,
      })
    ).toEqual({
      ok: true,
      query: {
        title: '用户管理',
        oper_name: 'admin',
        oper_ip: '10.0.0.5',
        business_type: 1,
        status: 0,
      },
    });
  });

  it('converts an inclusive local date range to UTC RFC3339', () => {
    const value = '2026-07-13T09:30';

    expect(
      toOperationLogQuery({
        ...DEFAULT_OPERATION_LOG_FILTERS,
        begin_time: value,
        end_time: value,
      })
    ).toEqual({
      ok: true,
      query: {
        begin_time: new Date(2026, 6, 13, 9, 30).toISOString(),
        end_time: new Date(2026, 6, 13, 9, 30).toISOString(),
      },
    });
  });
});

describe('operation log filter state', () => {
  it('keeps the last valid query when the visible date range is invalid', () => {
    const draft = {
      ...DEFAULT_OPERATION_LOG_FILTERS,
      begin_time: '2026-07-13T10:00',
      end_time: '2026-07-13T09:00',
    };

    expect(updateOperationLogFilterState({ oper_name: 'admin' }, draft)).toEqual({
      draft,
      query: { oper_name: 'admin' },
      error: AUDIT_FILTER_ERROR.INVALID_RANGE,
      resetTable: false,
    });
  });
});

describe('login log query conversion', () => {
  it('builds the full RuoYi login-log query', () => {
    expect(
      toLoginLogQuery({
        ...DEFAULT_LOGIN_LOG_FILTERS,
        ipaddr: ' 10.0.0.8 ',
        user_name: ' operator ',
        status: 1,
        event_type: 'refresh_failure',
        begin_time: '2026-07-13T08:00',
        end_time: '2026-07-13T18:00',
      })
    ).toEqual({
      ok: true,
      query: {
        ipaddr: '10.0.0.8',
        user_name: 'operator',
        status: 1,
        event_type: 'refresh_failure',
        begin_time: new Date(2026, 6, 13, 8, 0).toISOString(),
        end_time: new Date(2026, 6, 13, 18, 0).toISOString(),
      },
    });
  });
});
