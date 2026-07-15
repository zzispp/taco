export type CursorPageResponse<T> = Readonly<{
  items: T[];
  next_cursor: string | null;
  previous_cursor: string | null;
  has_next: boolean;
  has_previous: boolean;
}>;
