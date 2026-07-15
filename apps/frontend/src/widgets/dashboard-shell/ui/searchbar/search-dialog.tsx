import type { ChangeEvent } from 'react';
import type { OutputItem } from './utils';

import parse from 'autosuggest-highlight/parse';
import match from 'autosuggest-highlight/match';

import MenuList from '@mui/material/MenuList';
import { useTheme } from '@mui/material/styles';
import InputAdornment from '@mui/material/InputAdornment';
import Dialog, { dialogClasses } from '@mui/material/Dialog';
import MenuItem, { menuItemClasses } from '@mui/material/MenuItem';
import InputBase, { inputBaseClasses } from '@mui/material/InputBase';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { SearchNotFound } from 'src/shared/ui/search-not-found';

import { ResultItem } from './result-item';

type SearchDialogProps = {
  open: boolean;
  query: string;
  items: OutputItem[];
  onQueryChange: (query: string) => void;
  onClose: () => void;
};

export function SearchDialog(props: SearchDialogProps) {
  const theme = useTheme();
  const notFound = props.query.length > 0 && props.items.length === 0;

  return (
    <Dialog
      fullWidth
      maxWidth="sm"
      open={props.open}
      onClose={props.onClose}
      transitionDuration={{ enter: theme.transitions.duration.shortest, exit: 100 }}
      sx={{
        [`& .${dialogClasses.paper}`]: { mt: 15, overflow: 'unset' },
        [`& .${dialogClasses.container}`]: { alignItems: 'flex-start' },
      }}
    >
      <SearchInput open={props.open} query={props.query} onQueryChange={props.onQueryChange} />
      {notFound ? (
        <SearchNotFound query={props.query} sx={{ py: 15, px: 2.5 }} />
      ) : (
        <Scrollbar sx={{ p: 2.5, height: 400 }}>
          <SearchResults items={props.items} query={props.query} onClose={props.onClose} />
        </Scrollbar>
      )}
    </Dialog>
  );
}

function SearchInput({
  open,
  query,
  onQueryChange,
}: Pick<SearchDialogProps, 'open' | 'query' | 'onQueryChange'>) {
  const handleChange = (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
    onQueryChange(event.target.value);

  return (
    <InputBase
      fullWidth
      autoFocus={open}
      placeholder="Search..."
      value={query}
      onChange={handleChange}
      startAdornment={
        <InputAdornment position="start">
          <Iconify icon="eva:search-fill" width={24} sx={{ color: 'text.disabled' }} />
        </InputAdornment>
      }
      endAdornment={<Label sx={{ letterSpacing: 1, color: 'text.secondary' }}>esc</Label>}
      inputProps={{ id: 'search-input' }}
      sx={{
        p: 3,
        borderBottom: (theme) => `solid 1px ${theme.vars.palette.divider}`,
        [`& .${inputBaseClasses.input}`]: { typography: 'h6' },
      }}
    />
  );
}

function SearchResults({
  items,
  query,
  onClose,
}: Pick<SearchDialogProps, 'items' | 'query' | 'onClose'>) {
  return (
    <MenuList
      disablePadding
      sx={{
        [`& .${menuItemClasses.root}`]: {
          p: 0,
          mb: 0,
          '&:hover': { bgcolor: 'transparent' },
        },
      }}
    >
      {items.map((item) => (
        <MenuItem disableRipple key={`${item.title}${item.path}`}>
          <ResultItem
            path={highlight(item.path, query)}
            title={highlight(item.title, query)}
            href={item.path}
            labels={item.group.split('.')}
            onClick={onClose}
          />
        </MenuItem>
      ))}
    </MenuList>
  );
}

function highlight(value: string, query: string) {
  return parse(value, match(value, query, { insideWords: true }));
}
