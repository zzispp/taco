import { vi, it, expect, describe, beforeEach } from 'vitest';

import axios from 'src/shared/api/http-client';
import { downloadBlobResponse } from 'src/shared/api/download';

import { JOB_LOG_STATUS, schedulerEndpoints, SCHEDULER_TRIGGER_TYPE } from 'src/entities/scheduler';

import {
  exportJobLogs,
  schedulerJobRefreshEndpoints,
  schedulerJobDetailCacheMatcher,
} from './index';

vi.mock('src/shared/api/http-client', () => ({ default: { post: vi.fn() } }));
vi.mock('src/shared/api/download', () => ({ downloadBlobResponse: vi.fn() }));

beforeEach(() => vi.clearAllMocks());

describe('scheduler mutation cache capabilities', () => {
  it('does not revalidate importable tasks without import capability', () => {
    expect(schedulerJobRefreshEndpoints({ canRefreshImportableTasks: false })).toEqual([
      schedulerEndpoints.jobs,
    ]);
  });

  it('revalidates importable tasks when import capability is present', () => {
    expect(schedulerJobRefreshEndpoints({ canRefreshImportableTasks: true })).toEqual([
      schedulerEndpoints.jobs,
      schedulerEndpoints.importableJobs,
    ]);
  });

  it('matches every localized cache key for only the mutated job ids', () => {
    const matches = schedulerJobDetailCacheMatcher(['job-1', 'job-2']);

    expect(matches([schedulerEndpoints.job('job-1'), 'cn'])).toBe(true);
    expect(matches([schedulerEndpoints.job('job-2'), 'en'])).toBe(true);
    expect(matches([schedulerEndpoints.job('job-3'), 'cn'])).toBe(false);
    expect(matches([schedulerEndpoints.jobs, { params: {} }, 'cn'])).toBe(false);
  });
});

describe('scheduler log export', () => {
  it('sends the exact typed query used by the log list', async () => {
    const response = { data: new Blob(), headers: {} };
    vi.mocked(axios.post).mockResolvedValue(response as never);
    const query = {
      job_name: 'HTTP probe',
      job_group: 'SYSTEM',
      status: JOB_LOG_STATUS.FAILED,
      trigger_type: SCHEDULER_TRIGGER_TYPE.MANUAL,
      begin_time: '2026-07-10T10:00:00.000Z',
      end_time: '2026-07-10T11:00:00.000Z',
    };

    await exportJobLogs(query);

    expect(axios.post).toHaveBeenCalledExactlyOnceWith(schedulerEndpoints.jobLogsExport, null, {
      params: query,
      responseType: 'blob',
    });
    expect(downloadBlobResponse).toHaveBeenCalledExactlyOnceWith(response, 'job_logs.xlsx');
  });
});
