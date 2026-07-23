import { getFilePreviewBlob } from 'src/entities/file';

export type FilePreviewWindow = Readonly<{
  closed: boolean;
  close: () => void;
  location: Readonly<{ href: string; replace: (url: string) => void }>;
  addEventListener: (event: 'load', listener: () => void) => void;
  removeEventListener: (event: 'load', listener: () => void) => void;
}> & { opener: unknown };

export type FilePreviewRuntime = Readonly<{
  openBlank: () => FilePreviewWindow | null;
  createObjectURL: (blob: Blob) => string;
  revokeObjectURL: (url: string) => void;
}>;

export class FilePreviewWindowBlockedError extends Error {
  constructor() {
    super('file preview window was blocked');
    this.name = 'FilePreviewWindowBlockedError';
  }
}

export class FilePreviewWindowClosedError extends Error {
  constructor() {
    super('file preview window was closed');
    this.name = 'FilePreviewWindowClosedError';
  }
}

const browserPreviewRuntime: FilePreviewRuntime = {
  openBlank: () => window.open('', '_blank'),
  createObjectURL: (blob) => URL.createObjectURL(blob),
  revokeObjectURL: (url) => URL.revokeObjectURL(url),
};

// The opener keeps these references so lifecycle mutations can close materialized previews.
const previewTargets = new Map<string, Set<FilePreviewWindow>>();

export async function openFilePreview(
  fileId: string,
  runtime: FilePreviewRuntime = browserPreviewRuntime
) {
  const target = runtime.openBlank();
  if (!target) throw new FilePreviewWindowBlockedError();
  target.opener = null;
  registerPreviewTarget(fileId, target);
  try {
    const blob = await getFilePreviewBlob(fileId);
    if (target.closed) throw new FilePreviewWindowClosedError();
    installPreviewBlob(target, blob, runtime);
  } catch (error) {
    closePreviewTarget(target);
    throw error;
  } finally {
    if (target.closed) unregisterPreviewTarget(fileId, target);
  }
}

export function invalidateFilePreviews(fileIds: readonly string[]) {
  for (const fileId of new Set(fileIds)) {
    const targets = previewTargets.get(fileId);
    if (!targets) continue;
    previewTargets.delete(fileId);
    for (const target of targets) closePreviewTarget(target);
  }
}

function installPreviewBlob(target: FilePreviewWindow, blob: Blob, runtime: FilePreviewRuntime) {
  const objectUrl = runtime.createObjectURL(blob);
  const revoke = createOnce(() => runtime.revokeObjectURL(objectUrl));
  const handleLoad = () => {
    if (target.location.href !== objectUrl) return;
    target.removeEventListener('load', handleLoad);
    revoke();
  };
  try {
    target.addEventListener('load', handleLoad);
    target.location.replace(objectUrl);
  } catch (error) {
    target.removeEventListener('load', handleLoad);
    revoke();
    throw error;
  }
}

function registerPreviewTarget(fileId: string, target: FilePreviewWindow) {
  const targets = previewTargets.get(fileId) ?? new Set<FilePreviewWindow>();
  targets.add(target);
  previewTargets.set(fileId, targets);
}

function unregisterPreviewTarget(fileId: string, target: FilePreviewWindow) {
  const targets = previewTargets.get(fileId);
  if (!targets) return;
  targets.delete(target);
  if (!targets.size) previewTargets.delete(fileId);
}

function closePreviewTarget(target: FilePreviewWindow) {
  if (!target.closed) target.close();
}

function createOnce(action: () => void) {
  let completed = false;
  return () => {
    if (completed) return;
    completed = true;
    action();
  };
}
