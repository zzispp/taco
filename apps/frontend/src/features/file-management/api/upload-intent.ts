import { sha256Text } from '../lib/sha256';

const UPLOAD_INTENT_STORAGE_PREFIX = 'taco:file-upload:';
const memoryIntentKeys = new Map<string, string>();

export type UploadIntentIdentity = Readonly<{
  spaceId: string;
  parentId: string | null;
  fileName: string;
  fileSize: number;
  digest: string;
  contentType: string;
}>;

export function uploadIntentFingerprint(identity: UploadIntentIdentity): string {
  return sha256Text(
    JSON.stringify([
      identity.spaceId,
      identity.parentId,
      identity.fileName,
      identity.fileSize,
      identity.digest,
      identity.contentType,
    ])
  );
}

export function uploadIntentKey(fingerprint: string): string {
  const storage = uploadIntentStorage();
  const stored = storage
    ? storage.getItem(`${UPLOAD_INTENT_STORAGE_PREFIX}${fingerprint}`)
    : memoryIntentKeys.get(fingerprint);
  if (stored) return stored;
  const key = globalThis.crypto.randomUUID();
  if (storage) storage.setItem(`${UPLOAD_INTENT_STORAGE_PREFIX}${fingerprint}`, key);
  else memoryIntentKeys.set(fingerprint, key);
  return key;
}

export function clearUploadIntent(fingerprint: string) {
  const storage = uploadIntentStorage();
  if (storage) storage.removeItem(`${UPLOAD_INTENT_STORAGE_PREFIX}${fingerprint}`);
  else memoryIntentKeys.delete(fingerprint);
}

function uploadIntentStorage(): Storage | null {
  return typeof window === 'undefined' ? null : window.localStorage;
}
