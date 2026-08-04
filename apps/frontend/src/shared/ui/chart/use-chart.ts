import type { ChartOptions } from './types';

import { merge } from 'es-toolkit';

import { useTheme } from '@mui/material/styles';

import { createBaseChartOptions } from './chart-options';

// ----------------------------------------------------------------------

export function useChart(updatedOptions?: ChartOptions): ChartOptions {
  const theme = useTheme();

  return merge(createBaseChartOptions(theme), updatedOptions ?? {});
}
