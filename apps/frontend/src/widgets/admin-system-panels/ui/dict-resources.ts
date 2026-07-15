import type { DictType } from 'src/entities/system';

import { useMemo, useState } from 'react';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';
import { useLocalDateTimeFilterState } from 'src/shared/lib/use-local-date-time-filter-state';

import { useHasPermission } from 'src/entities/session';
import { useDictData, useDictTypes } from 'src/entities/system';

import { withSelectionHead } from 'src/widgets/admin-common';

import { dictDataHead, dictTypeHead } from './dict-helpers';
import { DEFAULT_DATA_FILTERS, DEFAULT_TYPE_FILTERS } from './dict-constants';

export function useDictResources(selected: DictType | null) {
  const { t } = useTranslate('admin');
  const typeTable = useTable({ defaultLimit: DEFAULT_TABLE_LIMIT });
  const typeFilters = useLocalDateTimeFilterState(DEFAULT_TYPE_FILTERS, {
    onValidQuery: typeTable.onResetCursor,
  });
  const [dataFilters, setDataFilters] = useState(DEFAULT_DATA_FILTERS);
  const dictTypes = useDictTypes(typeTable.cursorRequest, typeFilters.query);
  const activeType = selected?.dict_type ?? dictTypes.items[0]?.dict_type ?? '';
  const dataTable = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    scopeKey: JSON.stringify({ activeType, dataFilters }),
  });
  const dictData = useDictData(dataTable.cursorRequest, {
    ...dataFilters,
    dict_type: activeType,
  });
  const canAdd = useHasPermission('system:dict:add');
  const canRemove = useHasPermission('system:dict:remove');
  const canExport = useHasPermission('system:dict:export');
  const heads = useDictHeads(t, canRemove);

  return {
    t,
    typeTable,
    dataTable,
    typeFilters: typeFilters.draft,
    setTypeFilters: typeFilters.change,
    typeFilterQuery: typeFilters.query,
    typeFilterError: typeFilters.error,
    dataFilters,
    setDataFilters,
    dictTypes,
    activeType,
    dictData,
    canAdd,
    canRemove,
    canExport,
    ...heads,
  };
}

function useDictHeads(t: ReturnType<typeof useTranslate>['t'], canRemove: boolean) {
  const typeHead = useMemo(() => dictTypeHead(t), [t]);
  const dataHead = useMemo(() => dictDataHead(t), [t]);
  const loadingTypeHead = useMemo(
    () => (canRemove ? withSelectionHead(typeHead) : typeHead),
    [canRemove, typeHead]
  );
  const loadingDataHead = useMemo(
    () => (canRemove ? withSelectionHead(dataHead) : dataHead),
    [canRemove, dataHead]
  );

  return { typeHead, dataHead, loadingTypeHead, loadingDataHead };
}

export type DictResources = ReturnType<typeof useDictResources>;
