import type { SystemLogFilterQuery } from 'src/entities/system-log';

export type SystemLogCleanupPreview = Readonly<{
  query: SystemLogFilterQuery;
  count: number;
}>;

export function createSystemLogCleanupPreview(
  query: SystemLogFilterQuery,
  count: number
): SystemLogCleanupPreview {
  return Object.freeze({
    query: Object.freeze({ ...query }),
    count,
  });
}

export function acceptsSystemLogCleanupPreview(activeRevision: number, responseRevision: number) {
  return activeRevision === responseRevision;
}
