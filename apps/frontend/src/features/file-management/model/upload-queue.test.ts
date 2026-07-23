import { it, expect, describe } from 'vitest';

import { createUploadQueue, buildUploadQueueTree, truncateUploadDigest } from './upload-queue';

describe('upload queue', () => {
  it('accepts a regular dropped file without webkitRelativePath', () => {
    const dropped = new File(['report'], 'quarterly-report.pdf', { type: 'application/pdf' });
    Object.defineProperty(dropped, 'path', { value: './quarterly-report.pdf' });

    expect(dropped.webkitRelativePath).toBeUndefined();
    expect(createUploadQueue([dropped]).map((item) => item.relativePath)).toEqual([
      'quarterly-report.pdf',
    ]);
  });

  it('keeps dropped folder paths and renders them as a hierarchical table', () => {
    const queue = createUploadQueue([
      file('/Architecture/api/service.md', 'service'),
      file('/Architecture/readme.md', 'readme'),
      file('notes.txt', 'notes'),
    ]);

    expect(queue.map((item) => item.relativePath)).toEqual([
      'Architecture/api/service.md',
      'Architecture/readme.md',
      'notes.txt',
    ]);
    expect(buildUploadQueueTree(queue)).toEqual([
      expect.objectContaining({ kind: 'folder', name: 'Architecture', depth: 0, size: 13 }),
      expect.objectContaining({ kind: 'folder', name: 'api', depth: 1, size: 7 }),
      expect.objectContaining({ kind: 'file', name: 'service.md', depth: 2 }),
      expect.objectContaining({ kind: 'file', name: 'readme.md', depth: 1 }),
      expect.objectContaining({ kind: 'file', name: 'notes.txt', depth: 0 }),
    ]);
  });

  it('rejects a file path that could escape the selected folder', () => {
    expect(() => createUploadQueue([file('../outside.txt', 'outside')])).toThrow(
      'Invalid upload path'
    );
    expect(() => createUploadQueue([file('./../outside.txt', 'outside')])).toThrow(
      'Invalid upload path'
    );
  });

  it('keeps repeated file names as separate upload items', () => {
    const queue = createUploadQueue([file('report.txt', 'first'), file('report.txt', 'second')]);

    expect(buildUploadQueueTree(queue).filter((row) => row.kind === 'file')).toHaveLength(2);
  });

  it('keeps both ends of a SHA-256 digest in the table preview', () => {
    const digest = 'a'.repeat(32) + 'b'.repeat(32);

    expect(truncateUploadDigest(digest)).toBe(`${'a'.repeat(10)}...${'b'.repeat(8)}`);
  });
});

function file(path: string, content: string): File {
  const name = path.split('/').at(-1) ?? 'unnamed';
  const value = new File([content], name, { type: 'text/plain' });
  Object.defineProperty(value, 'path', { value: path });
  return value;
}
