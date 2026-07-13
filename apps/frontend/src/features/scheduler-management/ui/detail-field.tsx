import type { ReactNode } from 'react';

import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';

export function DetailField(props: { label: string; children: ReactNode }) {
  return (
    <Box
      sx={{
        py: 1.25,
        gap: 2,
        display: 'grid',
        borderBottom: 1,
        borderColor: 'divider',
        gridTemplateColumns: { xs: '1fr', sm: '180px minmax(0, 1fr)' },
      }}
    >
      <Typography component="dt" variant="subtitle2">
        {props.label}
      </Typography>
      <Typography component="dd" variant="body2" sx={{ m: 0, overflowWrap: 'anywhere' }}>
        {props.children}
      </Typography>
    </Box>
  );
}
