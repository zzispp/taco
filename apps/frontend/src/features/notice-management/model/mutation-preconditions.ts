export const NOTICE_DELETE_TARGET_REQUIRED_ERROR = 'A notice delete target is required';

export function requireNoticeDeleteTarget<T>(target: T | null): T {
  if (target === null) {
    throw new Error(NOTICE_DELETE_TARGET_REQUIRED_ERROR);
  }
  return target;
}
