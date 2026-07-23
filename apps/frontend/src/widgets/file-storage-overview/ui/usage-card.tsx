import type { CardProps } from '@mui/material/Card';

import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

export function UsageCard({
  label,
  value,
  sx,
  ...other
}: CardProps & { label: string; value: string }) {
  return (
    <Card variant="outlined" sx={[{ p: 2.5 }, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <Stack spacing={1}>
        <Typography variant="body2" color="text.secondary">
          {label}
        </Typography>
        <Typography variant="h5">{value}</Typography>
      </Stack>
    </Card>
  );
}
