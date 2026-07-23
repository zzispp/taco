import type { ReactElement } from 'react';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

import { ThemeProvider } from '@mui/material/styles';

import { createTheme } from 'src/shared/theme';

import { FileThumbnail } from '../../file-thumbnail';
import { SingleFilePreview } from './single-file-preview';

const TEST_THEME = createTheme();

const PREVIEW_URL = 'blob:image-preview';
const setPreviewUrl = vi.fn();

vi.mock('../../file-thumbnail/use-file-preview', () => ({
  useFilePreview: () => ({ previewUrl: PREVIEW_URL, setPreviewUrl }),
}));

function renderWithTheme(element: ReactElement) {
  return renderToStaticMarkup(createElement(ThemeProvider, { theme: TEST_THEME }, element));
}

describe('single file upload preview', () => {
  it.each([
    ['document.pdf', 'ic-pdf.svg'],
    ['archive.zip', 'ic-zip.svg'],
    ['notes.txt', 'ic-txt.svg'],
  ])('%s uses its file icon instead of a blob image', (file, icon) => {
    const markup = renderWithTheme(createElement(SingleFilePreview, { file }));

    expect(markup).toContain(`src="/assets/icons/files/${icon}"`);
    expect(markup).not.toContain(`src="${PREVIEW_URL}"`);
  });

  it('uses the object URL for an image preview', () => {
    const markup = renderWithTheme(createElement(SingleFilePreview, { file: 'photo.png' }));

    expect(markup).toContain(`src="${PREVIEW_URL}"`);
    expect(markup).not.toContain('/assets/icons/files/ic-img.svg');
  });

  it('uses the file metadata when the selected value is a browser File', () => {
    const file = new File(['%PDF-1.7'], 'document.pdf', { type: 'application/pdf' });
    const markup = renderWithTheme(createElement(SingleFilePreview, { file }));

    expect(markup).toContain('/assets/icons/files/ic-pdf.svg');
    expect(markup).not.toContain(`src="${PREVIEW_URL}"`);
  });

  it('keeps an explicit PDF blob URL out of image markup', () => {
    const markup = renderWithTheme(
      createElement(FileThumbnail, {
        file: 'document.pdf',
        showImage: true,
        previewUrl: 'blob:pdf-preview',
      })
    );

    expect(markup).toContain('/assets/icons/files/ic-pdf.svg');
    expect(markup).not.toContain('src="blob:pdf-preview"');
  });

  it('does not treat an explicit empty preview URL as a local file path', () => {
    const markup = renderWithTheme(
      createElement(FileThumbnail, {
        file: 'photo.png',
        showImage: true,
        previewUrl: '',
      })
    );

    expect(markup).toContain('/assets/icons/files/ic-img.svg');
    expect(markup).not.toContain('src="photo.png"');
    expect(markup).not.toContain(`src="${PREVIEW_URL}"`);
  });
});
