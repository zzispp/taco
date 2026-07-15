'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';

import { EmptyContent } from '../empty-content';

// ----------------------------------------------------------------------

export type TableNoDataProps = {
  notFound: boolean;
  colSpan: number;
  title?: string;
  sx?: SxProps<Theme>;
};

export function TableNoData({ notFound, colSpan, title, sx }: TableNoDataProps) {
  return (
    <TableRow>
      {notFound ? (
        <TableCell colSpan={colSpan}>
          <EmptyContent
            title={title}
            filled
            sx={[{ py: 10 }, ...(Array.isArray(sx) ? sx : [sx])]}
          />
        </TableCell>
      ) : (
        <TableCell colSpan={colSpan} sx={{ p: 0 }} />
      )}
    </TableRow>
  );
}
