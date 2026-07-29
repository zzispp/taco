export type CursorPageRefreshOptions = Readonly<{
  cursor: string | null;
  resetCursor: () => void;
  refresh: () => Promise<void>;
}>;

export async function refreshCursorPage(options: CursorPageRefreshOptions) {
  if (options.cursor) {
    options.resetCursor();
    return;
  }
  await options.refresh();
}
