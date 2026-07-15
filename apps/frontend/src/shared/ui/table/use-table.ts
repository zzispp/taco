'use client';

import type { UseTableProps, UseTableReturn } from './use-table-types';

import { useState, useCallback } from 'react';

import { DEFAULT_CURSOR_LIMIT } from 'src/shared/api/pagination';
import { useCursorNavigation } from 'src/shared/lib/use-cursor-navigation';

export type { UseTableProps, UseTableReturn } from './use-table-types';

export const DEFAULT_TABLE_LIMIT = DEFAULT_CURSOR_LIMIT;

export function useTable(props?: UseTableProps): UseTableReturn {
  const options = props ?? {};
  const state = useTableState(options);
  const { setDense, setOrder, setOrderBy, setSelected } = state;
  const navigation = useCursorNavigation(
    options.defaultLimit ?? DEFAULT_TABLE_LIMIT,
    options.scopeKey
  );
  const cursorActions = useTableCursorActions({ navigation, setSelected });
  const onSort = useTableSortAction({ state, resetCursor: cursorActions.onResetCursor });
  const selectionActions = useTableSelectionActions(setSelected);

  return {
    ...state,
    limit: navigation.limit,
    cursor: navigation.cursor,
    cursorRequest: navigation.request,
    visitedBatchIndex: navigation.visitedBatchIndex,
    onSort,
    ...cursorActions,
    ...selectionActions,
    onChangeDense: useCallback(
      (event: React.ChangeEvent<HTMLInputElement>) => setDense(event.target.checked),
      [setDense]
    ),
    setOrder,
    setOrderBy,
  };
}

function useTableState(options: UseTableProps) {
  const [dense, setDense] = useState(!!options.defaultDense);
  const [orderBy, setOrderBy] = useState(options.defaultOrderBy ?? 'name');
  const [order, setOrder] = useState<'asc' | 'desc'>(options.defaultOrder ?? 'asc');
  const [selected, setSelected] = useState<string[]>(options.defaultSelected ?? []);

  return { dense, orderBy, order, selected, setDense, setOrderBy, setOrder, setSelected };
}

type TableCursorActionsOptions = Readonly<{
  navigation: ReturnType<typeof useCursorNavigation>;
  setSelected: UseTableReturn['setSelected'];
}>;

function useTableCursorActions({ navigation, setSelected }: TableCursorActionsOptions) {
  const onResetCursor = useCallback(() => {
    setSelected([]);
    navigation.reset();
  }, [navigation, setSelected]);
  const onNextCursor = useCallback(
    (targetCursor: string | null) => {
      if (!targetCursor || targetCursor === navigation.cursor) return;
      setSelected([]);
      navigation.next(targetCursor);
    },
    [navigation, setSelected]
  );
  const onPreviousCursor = useCallback(
    (targetCursor: string | null) => {
      if (!targetCursor || targetCursor === navigation.cursor) return;
      setSelected([]);
      navigation.previous(targetCursor);
    },
    [navigation, setSelected]
  );
  const onChangeLimit = useCallback(
    (nextLimit: number) => {
      setSelected([]);
      navigation.changeLimit(nextLimit);
    },
    [navigation, setSelected]
  );

  return { onResetCursor, onNextCursor, onPreviousCursor, onChangeLimit };
}

type TableSortOptions = Readonly<{
  state: ReturnType<typeof useTableState>;
  resetCursor: () => void;
}>;

function useTableSortAction({ state, resetCursor }: TableSortOptions) {
  return useCallback(
    (id: string) => {
      if (!id) return;
      state.setOrder(state.orderBy === id && state.order === 'asc' ? 'desc' : 'asc');
      state.setOrderBy(id);
      resetCursor();
    },
    [resetCursor, state]
  );
}

function useTableSelectionActions(setSelected: UseTableReturn['setSelected']) {
  return {
    onSelectRow: useCallback(
      (id: string) => setSelected((current) => toggleSelected(current, id)),
      [setSelected]
    ),
    onSelectAllRows: useCallback(
      (checked: boolean, ids: string[]) => setSelected(checked ? [...ids] : []),
      [setSelected]
    ),
  };
}

function toggleSelected(selected: string[], id: string) {
  return selected.includes(id) ? selected.filter((value) => value !== id) : [...selected, id];
}
