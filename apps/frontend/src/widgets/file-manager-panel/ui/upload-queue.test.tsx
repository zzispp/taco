import type { UploadQueueItem } from 'src/features/file-management';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

vi.mock('src/shared/ui/scrollbar', () => ({
  Scrollbar: ({ children }: { children: React.ReactNode }) => createElement('div', null, children),
}));

vi.mock('react-dropzone', () => ({
  useDropzone: () => ({
    getRootProps: () => ({}),
    getInputProps: () => ({}),
    isDragActive: false,
  }),
}));

vi.mock('@mui/material/Tooltip', async () => {
  const { createElement: create } = await import('react');
  return {
    default: ({ title, children }: { title: string; children: React.ReactNode }) =>
      create('span', { 'data-tooltip': title }, children),
  };
});

import { UploadQueue } from './upload-queue';

describe('upload queue table', () => {
  it('renders a folder upload as tree rows with a truncated and inspectable digest', () => {
    const digest = 'a'.repeat(32) + 'b'.repeat(32);
    const html = renderToStaticMarkup(
      createElement(UploadQueue, {
        items: [item('Architecture/api/service.md', digest)],
        disabled: false,
        onAppend: vi.fn(),
        onRemove: vi.fn(),
      })
    );

    expect(html).toContain('Architecture');
    expect(html).toContain('api');
    expect(html).toContain('service.md');
    expect(html).toContain('text/plain');
    expect(html).toContain('aaaaaaaaaa...bbbbbbbb');
    expect(html).toContain(`data-tooltip="${digest}"`);
    expect(html).not.toContain('minimal__upload__preview__single');
  });
});

function item(relativePath: string, digest: string): UploadQueueItem {
  const file = new File(['service'], 'service.md', { type: 'text/plain' });
  return {
    id: 'service',
    file,
    relativePath,
    digest,
    progress: null,
    status: 'queued',
  };
}
