import type { UploadProgress } from '../model/upload-progress';

export type UploadInput = Readonly<{
  space_id: string;
  parent_id: string | null;
}>;

export type UploadOptions = Readonly<{
  signal?: AbortSignal;
  digest?: string;
  onProgress?: (progress: UploadProgress) => void;
}>;
