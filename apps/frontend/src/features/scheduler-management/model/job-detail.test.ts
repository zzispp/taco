import { it, expect, describe } from 'vitest';

import {
  JOB_DETAIL_TAB,
  isCurrentJobDetail,
  formatTaskParameters,
  jobDetailDisplayValue,
  EMPTY_JOB_DETAIL_VALUE,
  formatRuntimeErrorDetail,
} from './job-detail';

describe('job detail model', () => {
  it('defines the four detail tabs in display order', () => {
    expect(Object.values(JOB_DETAIL_TAB)).toEqual([
      'configuration',
      'schedule',
      'method',
      'metadata',
    ]);
  });

  it('rejects stale detail data from a previous selection', () => {
    expect(isCurrentJobDetail({ job_id: 'job-2' }, { job_id: 'job-1' })).toBe(false);
    expect(isCurrentJobDetail({ job_id: 'job-2' }, { job_id: 'job-2' })).toBe(true);
    expect(isCurrentJobDetail(null, { job_id: 'job-2' })).toBe(false);
  });

  it('serializes complete nested task parameters without truncation', () => {
    const body = 'x'.repeat(40_000);
    const content = formatTaskParameters({ body, nested: { enabled: true, value: null } });

    expect(JSON.parse(content)).toEqual({ body, nested: { enabled: true, value: null } });
    expect(content).toContain(body);
  });

  it('uses an explicit marker for nullable or blank detail values', () => {
    expect(jobDetailDisplayValue(null)).toBe(EMPTY_JOB_DETAIL_VALUE);
    expect(jobDetailDisplayValue('')).toBe(EMPTY_JOB_DETAIL_VALUE);
    expect(jobDetailDisplayValue('  ')).toBe(EMPTY_JOB_DETAIL_VALUE);
    expect(jobDetailDisplayValue('admin')).toBe('admin');
  });

  it('keeps the localized runtime diagnostic without duplicating equal text', () => {
    expect(formatRuntimeErrorDetail('Invalid cron', 'Invalid cron')).toBe('Invalid cron');
    expect(formatRuntimeErrorDetail('Invalid cron', 'Cron field 6 is invalid')).toBe(
      'Invalid cron: Cron field 6 is invalid'
    );
  });
});
