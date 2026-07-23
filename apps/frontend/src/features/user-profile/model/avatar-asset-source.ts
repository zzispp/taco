import { getFilePreviewBlob } from 'src/entities/file';

export function loadAvatarAssetPreview(id: string, signal: AbortSignal) {
  return getFilePreviewBlob(id, signal);
}
