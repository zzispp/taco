import { it, expect, describe } from 'vitest';

import {
  NOTICE_TYPE,
  NOTICE_STATUS,
  noticeTypeColors,
  noticeStatusColors,
  noticeTypeTranslationKeys,
  noticeStatusTranslationKeys,
} from './constants';

describe('notice constants', () => {
  it('has one translation key for every notice type', () => {
    expect(Object.keys(noticeTypeTranslationKeys).sort()).toEqual(
      Object.values(NOTICE_TYPE).sort()
    );
  });

  it('has one translation key for every notice status', () => {
    expect(Object.keys(noticeStatusTranslationKeys).sort()).toEqual(
      Object.values(NOTICE_STATUS).sort()
    );
  });

  it('uses the RuoYi notice type colors', () => {
    expect(noticeTypeColors).toEqual({ '1': 'warning', '2': 'success' });
  });

  it('uses the RuoYi notice status colors', () => {
    expect(noticeStatusColors).toEqual({ '0': 'primary', '1': 'error' });
  });
});
