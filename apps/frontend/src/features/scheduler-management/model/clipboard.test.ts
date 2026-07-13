import { vi, it, expect, describe } from 'vitest';

import { copyTextToClipboard, copyTextWithFeedback } from './clipboard';

describe('copyTextToClipboard', () => {
  it('writes the complete execution id', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);

    await copyTextToClipboard('execution-123', { writeText });

    expect(writeText).toHaveBeenCalledExactlyOnceWith('execution-123');
  });

  it('propagates clipboard rejection', async () => {
    const failure = new Error('clipboard denied');
    const writeText = vi.fn().mockRejectedValue(failure);

    await expect(copyTextToClipboard('execution-123', { writeText })).rejects.toBe(failure);
  });

  it('reports success only after the complete id is written', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    const success = vi.fn();
    const failure = vi.fn();

    const copied = await copyTextWithFeedback('execution-123', { success, failure }, { writeText });

    expect(copied).toBe(true);
    expect(writeText).toHaveBeenCalledExactlyOnceWith('execution-123');
    expect(success).toHaveBeenCalledOnce();
    expect(failure).not.toHaveBeenCalled();
  });

  it('reports clipboard rejection without claiming success', async () => {
    const writeText = vi.fn().mockRejectedValue(new Error('clipboard denied'));
    const success = vi.fn();
    const failure = vi.fn();

    const copied = await copyTextWithFeedback('execution-123', { success, failure }, { writeText });

    expect(copied).toBe(false);
    expect(success).not.toHaveBeenCalled();
    expect(failure).toHaveBeenCalledOnce();
  });
});
