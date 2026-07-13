import type { DictResources } from './dict-resources';
import type { DictManagementState } from './dict-controller';

import { useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import { systemMutations } from 'src/features/system-management';

type DictToolOptions = {
  resources: DictResources;
  state: DictManagementState;
};

export function useDictToolActions({ resources, state }: DictToolOptions) {
  const refreshCache = useRefreshDictCache(resources.t);
  const exportTypes = useExportDictTypes(resources);
  const exportData = useExportDictData(resources);
  const toggleAllTypes = useCallback(
    (checked: boolean) => {
      state.setSelectedTypeIds(
        checked ? resources.dictTypes.items.map((item) => item.dict_id) : []
      );
    },
    [resources.dictTypes.items, state]
  );
  const toggleAllData = useCallback(
    (checked: boolean) => {
      state.setSelectedDataIds(
        checked ? resources.dictData.items.map((item) => item.dict_code) : []
      );
    },
    [resources.dictData.items, state]
  );

  return { refreshCache, exportTypes, exportData, toggleAllTypes, toggleAllData };
}

function useRefreshDictCache(t: DictResources['t']) {
  return useCallback(async () => {
    try {
      await systemMutations.refreshDictCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  }, [t]);
}

function useExportDictTypes(resources: DictResources) {
  return useCallback(async () => {
    if (resources.typeFilterError) {
      toast.error(
        resources.t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[resources.typeFilterError])
      );
      return;
    }
    try {
      await systemMutations.exportDictTypes(resources.typeFilterQuery);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : resources.t('messages.exportFailed'));
    }
  }, [resources]);
}

function useExportDictData(resources: DictResources) {
  return useCallback(async () => {
    if (!resources.activeType) return;
    try {
      await systemMutations.exportDictData({
        ...resources.dataFilters,
        dict_type: resources.activeType,
      });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : resources.t('messages.exportFailed'));
    }
  }, [resources]);
}
