'use client';

import type { MouseEvent as ReactMouseEvent } from 'react';

import { useState, useCallback } from 'react';

export type EntryContextMenuPosition = Readonly<{
  mouseX: number;
  mouseY: number;
}>;

type BrowserContextMenuEvent = Pick<
  ReactMouseEvent<HTMLElement>,
  'preventDefault' | 'stopPropagation'
>;

export function entryContextMenuPosition(
  event: Pick<ReactMouseEvent<HTMLElement>, 'clientX' | 'clientY'>
): EntryContextMenuPosition {
  return { mouseX: event.clientX, mouseY: event.clientY };
}

export function preventBrowserContextMenu(event: BrowserContextMenuEvent) {
  event.preventDefault();
  event.stopPropagation();
}

export function useEntryContextMenu() {
  const [position, setPosition] = useState<EntryContextMenuPosition | null>(null);

  const onContextMenu = useCallback((event: ReactMouseEvent<HTMLElement>) => {
    preventBrowserContextMenu(event);
    setPosition(entryContextMenuPosition(event));
  }, []);

  const close = useCallback(() => setPosition(null), []);

  return { position, onContextMenu, close };
}
