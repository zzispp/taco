'use client';

import { useSearchParams } from 'src/shared/routes/hooks';

import { parseFileManagerRoute } from './file-manager-route';

export function useFileManagerRoute() {
  return parseFileManagerRoute(useSearchParams());
}
