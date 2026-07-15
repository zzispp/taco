import type { Theme, SxProps, CSSObject } from '@mui/material/styles';

import Box from '@mui/material/Box';
import TableRow from '@mui/material/TableRow';
import Checkbox from '@mui/material/Checkbox';
import TableHead from '@mui/material/TableHead';
import TableCell from '@mui/material/TableCell';
import TableSortLabel from '@mui/material/TableSortLabel';

// ----------------------------------------------------------------------

const visuallyHidden: CSSObject = {
  border: 0,
  padding: 0,
  width: '1px',
  height: '1px',
  margin: '-1px',
  overflow: 'hidden',
  position: 'absolute',
  whiteSpace: 'nowrap',
  clip: 'rect(0 0 0 0)',
};

// ----------------------------------------------------------------------

export type TableHeadCellProps = {
  id: string;
  label?: string;
  sortable?: boolean;
  width?: CSSObject['width'];
  align?: 'left' | 'center' | 'right';
  sx?: SxProps<Theme>;
};

type SortOrder = 'asc' | 'desc';

export type TableHeadCustomProps = {
  selectAllRowsLabel: string;
  sortStatusLabel: (order: SortOrder) => string;
  orderBy?: string;
  rowCount?: number;
  sx?: SxProps<Theme>;
  numSelected?: number;
  order?: SortOrder;
  headCells: TableHeadCellProps[];
  onSort?: (id: string) => void;
  onSelectAllRows?: (checked: boolean) => void;
};

export function TableHeadCustom({
  sx,
  order,
  onSort,
  orderBy,
  headCells,
  rowCount = 0,
  numSelected = 0,
  onSelectAllRows,
  selectAllRowsLabel,
  sortStatusLabel,
}: TableHeadCustomProps) {
  return (
    <TableHead sx={sx}>
      <TableRow>
        {onSelectAllRows ? (
          <SelectAllCell
            rowCount={rowCount}
            numSelected={numSelected}
            label={selectAllRowsLabel}
            onSelectAllRows={onSelectAllRows}
          />
        ) : null}
        {headCells.map((headCell) => (
          <TableHeadCell
            key={headCell.id}
            headCell={headCell}
            order={order}
            orderBy={orderBy}
            onSort={onSort}
            sortStatusLabel={sortStatusLabel}
          />
        ))}
      </TableRow>
    </TableHead>
  );
}

function SelectAllCell({
  rowCount,
  numSelected,
  label,
  onSelectAllRows,
}: Readonly<{
  rowCount: number;
  numSelected: number;
  label: string;
  onSelectAllRows: (checked: boolean) => void;
}>) {
  return (
    <TableCell padding="checkbox">
      <Checkbox
        indeterminate={!!numSelected && numSelected < rowCount}
        checked={!!rowCount && numSelected === rowCount}
        onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
          onSelectAllRows(event.target.checked)
        }
        slotProps={{
          input: {
            id: 'all-row-checkbox',
            'aria-label': label,
          },
        }}
      />
    </TableCell>
  );
}

function TableHeadCell({
  headCell,
  order,
  orderBy,
  onSort,
  sortStatusLabel,
}: Readonly<{
  headCell: TableHeadCellProps;
  order?: SortOrder;
  orderBy?: string;
  onSort?: (id: string) => void;
  sortStatusLabel: (order: SortOrder) => string;
}>) {
  const active = orderBy === headCell.id;
  const sortable = Boolean(onSort && headCell.sortable !== false);
  const sortOrder = order ?? 'asc';
  return (
    <TableCell
      align={headCell.align || 'left'}
      sortDirection={active ? order : false}
      sx={[
        { width: headCell.width },
        ...(Array.isArray(headCell.sx) ? headCell.sx : [headCell.sx]),
      ]}
    >
      {sortable ? (
        <TableSortLabel
          hideSortIcon
          active={active}
          direction={active ? sortOrder : 'asc'}
          onClick={() => onSort?.(headCell.id)}
        >
          {headCell.label}
          {active ? (
            <Box component="span" sx={visuallyHidden}>
              {sortStatusLabel(sortOrder)}
            </Box>
          ) : null}
        </TableSortLabel>
      ) : (
        headCell.label
      )}
    </TableCell>
  );
}
