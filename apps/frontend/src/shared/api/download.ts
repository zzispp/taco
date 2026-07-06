import type { AxiosResponse } from 'axios';

const DISPOSITION_FILENAME_RE = /filename\*?=(?:UTF-8'')?"?([^";]+)"?/i;

export function downloadBlobResponse(response: AxiosResponse<Blob>, defaultFileName: string) {
  const fileName =
    fileNameFromDisposition(response.headers['content-disposition']) ?? defaultFileName;
  const url = URL.createObjectURL(response.data);
  const link = document.createElement('a');
  link.href = url;
  link.download = decodeFileName(fileName);
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

function fileNameFromDisposition(disposition: unknown) {
  if (typeof disposition !== 'string') return null;
  const match = DISPOSITION_FILENAME_RE.exec(disposition);
  return match?.[1] ?? null;
}

function decodeFileName(value: string) {
  return decodeURIComponent(value);
}
