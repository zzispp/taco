import type { Theme } from '@mui/material/styles';
import type { CardProps } from '@mui/material/Card';
import type { ChartOptions } from 'src/shared/ui/chart';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { PaletteColorKey } from 'src/shared/theme/core';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import { useTheme } from '@mui/material/styles';

import { CONFIG } from 'src/shared/config';
import { Iconify } from 'src/shared/ui/iconify';
import { SvgColor } from 'src/shared/ui/svg-color';
import { Chart, useChart } from 'src/shared/ui/chart';

import { formatNumber, formatPercent } from './format';

export type SummaryCardProps = CardProps & {
  title: string;
  value: string;
  percent: number;
  icon: IconifyName;
  color?: PaletteColorKey;
  chart: { categories: string[]; series: number[]; options?: ChartOptions };
};

export function SummaryCard(props: SummaryCardProps) {
  const { title, value, percent, icon, chart, color = 'primary', sx } = props;
  const theme = useTheme();
  const chartOptions = useChart(summaryChartOptions(theme, color, chart));

  return (
    <Card sx={[summaryCardSx(theme, color), ...(Array.isArray(sx) ? sx : [sx])]}>
      <SummaryIcon color={color} icon={icon} theme={theme} />

      <SummaryTrend color={color} percent={percent} />

      <Box
        sx={{
          display: 'flex',
          flexWrap: 'wrap',
          alignItems: 'flex-end',
          justifyContent: 'flex-end',
        }}
      >
        <Box sx={{ flexGrow: 1, minWidth: 112 }}>
          <Box sx={{ mb: 1, typography: 'subtitle2' }}>{title}</Box>
          <Box sx={{ typography: 'h4' }}>{value}</Box>
        </Box>

        <Chart
          type="line"
          series={[{ data: chart.series }]}
          options={chartOptions}
          sx={{ width: 84, height: 56 }}
        />
      </Box>

      <SummaryShape color={color} />
    </Card>
  );
}

function SummaryIcon(props: { color: PaletteColorKey; icon: IconifyName; theme: Theme }) {
  return (
    <Box sx={summaryIconSx(props.theme, props.color)}>
      <Iconify icon={props.icon} width={28} aria-hidden />
    </Box>
  );
}

function SummaryTrend(props: { color: PaletteColorKey; percent: number }) {
  return (
    <Box
      sx={{
        top: 16,
        gap: 0.5,
        right: 16,
        display: 'flex',
        position: 'absolute',
        alignItems: 'center',
      }}
    >
      <Iconify
        width={20}
        icon={props.percent < 0 ? 'eva:trending-down-fill' : 'eva:trending-up-fill'}
      />
      <Box component="span" sx={{ typography: 'subtitle2' }}>
        {props.percent > 0 && '+'}
        {formatPercent(props.percent)}
      </Box>
    </Box>
  );
}

function SummaryShape({ color }: { color: PaletteColorKey }) {
  return (
    <SvgColor
      src={`${CONFIG.assetsDir}/assets/background/shape-square.svg`}
      sx={{
        top: 0,
        left: -20,
        width: 240,
        zIndex: -1,
        height: 240,
        opacity: 0.24,
        position: 'absolute',
        color: `${color}.main`,
      }}
    />
  );
}

function summaryChartOptions(
  theme: Theme,
  color: PaletteColorKey,
  chart: SummaryCardProps['chart']
) {
  return {
    chart: { sparkline: { enabled: true } },
    colors: [theme.palette[color].dark],
    xaxis: { categories: chart.categories },
    yaxis: { show: false, labels: { show: false } },
    grid: { padding: { top: 6, left: 6, right: 6, bottom: 6 } },
    markers: { strokeWidth: 0 },
    tooltip: {
      y: { formatter: (value: number) => formatNumber(value), title: { formatter: () => '' } },
    },
    ...chart.options,
  };
}

function summaryIconSx(theme: Theme, color: PaletteColorKey) {
  return {
    mb: 3,
    width: 48,
    height: 48,
    display: 'flex',
    borderRadius: 2,
    alignItems: 'center',
    justifyContent: 'center',
    color: `${color}.dark`,
    backgroundColor: varAlpha(theme.vars.palette[color].mainChannel, 0.16),
    boxShadow: `inset 0 0 0 1px ${varAlpha(theme.vars.palette[color].mainChannel, 0.24)}`,
  };
}

function summaryCardSx(theme: Theme, color: PaletteColorKey) {
  return {
    p: 3,
    boxShadow: 'none',
    position: 'relative',
    color: `${color}.darker`,
    backgroundColor: 'common.white',
    backgroundImage: `linear-gradient(135deg, ${varAlpha(theme.vars.palette[color].lighterChannel, 0.48)}, ${varAlpha(theme.vars.palette[color].lightChannel, 0.48)})`,
  };
}
