'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Checkbox from '@mui/material/Checkbox';
import Typography from '@mui/material/Typography';

// ----------------------------------------------------------------------

export type TableSelectedActionProps = BoxProps & {
  dense?: boolean;
  rowCount: number;
  numSelected: number;
  action?: React.ReactNode;
  onSelectAllRows: (checked: boolean) => void;
};

const STANDARD_SELECTED_ACTION_HEIGHT = 58;
const DENSE_SELECTED_ACTION_HEIGHT = 38;

export function TableSelectedAction({
  sx,
  dense,
  action,
  rowCount,
  numSelected,
  onSelectAllRows,
  ...other
}: TableSelectedActionProps) {
  if (!numSelected) {
    return null;
  }

  const panelProps = { sx, dense, action, rowCount, numSelected, onSelectAllRows, ...other };

  return <SelectedActionPanel {...panelProps} />;
}

function SelectedActionPanel({
  sx,
  dense,
  action,
  rowCount,
  numSelected,
  onSelectAllRows,
  ...other
}: TableSelectedActionProps) {
  return (
    <Box sx={[getSelectedActionStyles(dense), ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <SelectedActionCheckbox
        rowCount={rowCount}
        numSelected={numSelected}
        onSelectAllRows={onSelectAllRows}
      />
      <SelectedActionLabel dense={dense} numSelected={numSelected} />
      {action && action}
    </Box>
  );
}

function getSelectedActionStyles(dense?: boolean) {
  return {
    pl: 1,
    pr: 2,
    top: 0,
    left: 0,
    width: 1,
    zIndex: 9,
    height: dense ? DENSE_SELECTED_ACTION_HEIGHT : STANDARD_SELECTED_ACTION_HEIGHT,
    display: 'flex',
    position: 'absolute',
    alignItems: 'center',
    bgcolor: 'primary.lighter',
  };
}

function SelectedActionCheckbox({
  rowCount,
  numSelected,
  onSelectAllRows,
}: Pick<TableSelectedActionProps, 'rowCount' | 'numSelected' | 'onSelectAllRows'>) {
  return (
    <Checkbox
      indeterminate={numSelected < rowCount}
      checked={Boolean(rowCount) && numSelected === rowCount}
      onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
        onSelectAllRows(event.target.checked)
      }
      slotProps={{ input: { id: 'deselect-all-checkbox', 'aria-label': 'Deselect all checkbox' } }}
    />
  );
}

function SelectedActionLabel({
  dense,
  numSelected,
}: Pick<TableSelectedActionProps, 'dense' | 'numSelected'>) {
  return (
    <Typography variant="subtitle2" sx={{ ml: dense ? 3 : 2, flexGrow: 1, color: 'primary.main' }}>
      {numSelected} selected
    </Typography>
  );
}
