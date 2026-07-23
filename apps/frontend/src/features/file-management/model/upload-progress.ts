export type UploadProgress = Readonly<{
  phase: 'hashing' | 'uploading' | 'completing';
  completedBytes: number;
  totalBytes: number;
  partNumber?: number;
  partCount?: number;
}>;

export function uploadProgressPercent(progress: UploadProgress): number {
  if (progress.totalBytes <= 0) return 0;
  return Math.min(100, Math.round((progress.completedBytes / progress.totalBytes) * 100));
}
