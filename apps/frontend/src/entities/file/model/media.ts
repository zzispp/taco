import type { FileEntry } from './types';

const THUMBNAIL_MIME_PATTERN = /^image\/(?:png|jpeg|webp|gif)$/;

export function supportsFileThumbnail(entry: Pick<FileEntry, 'type' | 'mime_type'>): boolean {
  return (
    entry.type === 'file' &&
    entry.mime_type !== null &&
    THUMBNAIL_MIME_PATTERN.test(entry.mime_type)
  );
}
