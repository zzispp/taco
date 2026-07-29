import type { DictResources } from './dict-resources';

import { useCallback } from 'react';

import { refreshCursorPage } from 'src/shared/api/refresh-cursor-page';

export function useDictPageRefresh(resources: DictResources) {
  return useCallback(async () => {
    await Promise.all([
      refreshCursorPage({
        cursor: resources.typeTable.cursor,
        resetCursor: resources.typeTable.onResetCursor,
        refresh: resources.dictTypes.refresh,
      }),
      refreshCursorPage({
        cursor: resources.dataTable.cursor,
        resetCursor: resources.dataTable.onResetCursor,
        refresh: resources.dictData.refresh,
      }),
    ]);
  }, [resources]);
}
