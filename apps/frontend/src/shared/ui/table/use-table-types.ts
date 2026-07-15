import type { CursorPageRequest } from 'src/shared/api/pagination';

export type UseTableReturn = {
  dense: boolean;
  limit: number;
  cursor: string | null;
  cursorRequest: CursorPageRequest;
  visitedBatchIndex: number;
  order: 'asc' | 'desc';
  orderBy: string;
  selected: string[];
  onSelectRow: (id: string) => void;
  onSelectAllRows: (checked: boolean, ids: string[]) => void;
  onResetCursor: () => void;
  onNextCursor: (cursor: string | null) => void;
  onPreviousCursor: (cursor: string | null) => void;
  onChangeLimit: (limit: number) => void;
  onSort: (id: string) => void;
  onChangeDense: (event: React.ChangeEvent<HTMLInputElement>) => void;
  setDense: React.Dispatch<React.SetStateAction<boolean>>;
  setOrderBy: React.Dispatch<React.SetStateAction<string>>;
  setSelected: React.Dispatch<React.SetStateAction<string[]>>;
  setOrder: React.Dispatch<React.SetStateAction<'desc' | 'asc'>>;
};

export type UseTableProps = {
  defaultDense?: boolean;
  defaultOrderBy?: string;
  defaultSelected?: string[];
  defaultLimit?: number;
  defaultOrder?: 'asc' | 'desc';
  scopeKey?: string;
};
