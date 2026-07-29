import type { CursorPageRefreshOptions } from 'src/shared/api/refresh-cursor-page';

import { refreshCursorPage } from 'src/shared/api/refresh-cursor-page';

type FileSpacePageRefreshOptions = CursorPageRefreshOptions &
  Readonly<{ refreshProviders: () => Promise<unknown> }>;

export async function refreshFileSpacePage(options: FileSpacePageRefreshOptions) {
  await Promise.all([refreshCursorPage(options), options.refreshProviders()]);
}
