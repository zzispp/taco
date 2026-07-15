'use client';

import type { BoxProps } from '@mui/material/Box';
import type { NavSectionProps } from 'src/shared/ui/nav-section';

import { useBoolean } from 'minimal-shared/hooks';
import { useMemo, useState, useEffect, useCallback } from 'react';

import { SearchDialog } from './search-dialog';
import { SearchButton } from './search-button';
import { applyFilter, flattenNavSections } from './utils';

export type SearchbarProps = BoxProps & {
  data?: NavSectionProps['data'];
};

export function Searchbar({ data: navItems = [], ...buttonProps }: SearchbarProps) {
  const controller = useSearchbar(navItems);

  return (
    <>
      <SearchButton {...buttonProps} onOpen={controller.onOpen} />
      <SearchDialog
        open={controller.open}
        query={controller.query}
        items={controller.items}
        onQueryChange={controller.onQueryChange}
        onClose={controller.onClose}
      />
    </>
  );
}

function useSearchbar(navItems: NavSectionProps['data']) {
  const { value: open, onFalse, onTrue, onToggle } = useBoolean();
  const [query, setQuery] = useState('');
  const allItems = useMemo(() => flattenNavSections(navItems), [navItems]);
  const items = useMemo(() => applyFilter({ inputData: allItems, query }), [allItems, query]);
  const onClose = useCallback(() => {
    onFalse();
    setQuery('');
  }, [onFalse]);
  const onKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!event.metaKey || event.key.toLowerCase() !== 'k') return;
      event.preventDefault();
      onToggle();
      setQuery('');
    },
    [onToggle]
  );

  useEffect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onKeyDown]);

  return { open, query, items, onOpen: onTrue, onClose, onQueryChange: setQuery };
}
