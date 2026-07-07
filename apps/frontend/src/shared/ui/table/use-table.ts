'use client';

import type { UseTableProps, UseTableReturn } from './use-table-types';

import { useState, useCallback } from 'react';

export type { UseTableProps, UseTableReturn } from './use-table-types';

export function useTable(props?: UseTableProps): UseTableReturn {
  const state = useTableState(props);
  const sort = useTableSort(state);
  const selection = useTableSelection(state);
  const pagination = useTablePagination(state, selection.selected.length);

  return {
    ...state,
    ...sort,
    ...selection,
    ...pagination,
  };
}

function useTableState(props?: UseTableProps) {
  const [dense, setDense] = useState(!!props?.defaultDense);
  const [page, setPage] = useState(props?.defaultCurrentPage ?? 0);
  const [orderBy, setOrderBy] = useState(props?.defaultOrderBy ?? 'name');
  const [rowsPerPage, setRowsPerPage] = useState(props?.defaultRowsPerPage ?? 5);
  const [order, setOrder] = useState<'asc' | 'desc'>(props?.defaultOrder ?? 'asc');
  const [selected, setSelected] = useState<string[]>(props?.defaultSelected ?? []);

  return { dense, page, orderBy, rowsPerPage, order, selected, setDense, setPage, setOrderBy, setRowsPerPage, setOrder, setSelected };
}

function useTableSort(state: ReturnType<typeof useTableState>) {
  const onSort = useCallback(
    (id: string) => {
      const isAsc = state.orderBy === id && state.order === 'asc';
      if (id !== '') {
        state.setOrder(isAsc ? 'desc' : 'asc');
        state.setOrderBy(id);
      }
    },
    [state]
  );

  return { onSort };
}

function useTableSelection(state: ReturnType<typeof useTableState>) {
  const onSelectRow = useCallback(
    (inputValue: string) => {
      state.setSelected(toggleSelected(state.selected, inputValue));
    },
    [state]
  );

  const onSelectAllRows = useCallback(
    (checked: boolean, inputValue: string[]) => {
      state.setSelected(checked ? inputValue : []);
    },
    [state]
  );

  return { selected: state.selected, onSelectRow, onSelectAllRows };
}

function useTablePagination(state: ReturnType<typeof useTableState>, totalSelected: number) {
  const onChangeRowsPerPage = useCallback((event: React.ChangeEvent<HTMLInputElement>) => {
    state.setPage(0);
    state.setRowsPerPage(parseInt(event.target.value, 10));
  }, [state]);

  const onUpdatePageDeleteRows = useCallback(
    (totalRowsInPage: number, totalRowsFiltered: number) => {
      state.setSelected([]);
      state.setPage(nextPageAfterRowsDelete({ currentPage: state.page, rowsPerPage: state.rowsPerPage, totalRowsInPage, totalRowsFiltered, totalSelected }));
    },
    [state, totalSelected]
  );

  return {
    onResetPage: useCallback(() => state.setPage(0), [state]),
    onChangeDense: useCallback((event: React.ChangeEvent<HTMLInputElement>) => state.setDense(event.target.checked), [state]),
    onChangePage: useCallback((event: unknown, newPage: number) => state.setPage(newPage), [state]),
    onChangeRowsPerPage,
    onUpdatePageDeleteRow: useCallback((totalRowsInPage: number) => state.setPage(nextPageAfterRowDelete(state.page, totalRowsInPage)), [state]),
    onUpdatePageDeleteRows,
  };
}

function toggleSelected(selected: string[], inputValue: string) {
  return selected.includes(inputValue) ? selected.filter((value) => value !== inputValue) : [...selected, inputValue];
}

function nextPageAfterRowDelete(page: number, totalRowsInPage: number) {
  return page && totalRowsInPage < 2 ? page - 1 : page;
}

type DeleteRowsState = {
  currentPage: number;
  rowsPerPage: number;
  totalRowsInPage: number;
  totalRowsFiltered: number;
  totalSelected: number;
};

function nextPageAfterRowsDelete(state: DeleteRowsState) {
  if (!state.currentPage) return state.currentPage;
  if (state.totalSelected === state.totalRowsInPage) return state.currentPage - 1;
  if (state.totalSelected === state.totalRowsFiltered) return 0;
  if (state.totalSelected > state.totalRowsInPage) return Math.ceil((state.totalRowsFiltered - state.totalSelected) / state.rowsPerPage) - 1;
  return state.currentPage;
}
