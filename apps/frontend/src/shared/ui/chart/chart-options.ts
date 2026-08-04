import type { Theme } from '@mui/material/styles';
import type { ChartOptions } from './types';

import { varAlpha } from 'minimal-shared/utils';

// ----------------------------------------------------------------------

export function createBaseChartOptions(theme: Theme): ChartOptions {
  return {
    chart: createChartConfig(theme),
    colors: createChartColors(theme),
    states: createChartStates(),
    fill: createChartFill(),
    dataLabels: { enabled: false },
    stroke: { width: 2.5, curve: 'smooth', lineCap: 'round' },
    grid: createChartGrid(theme),
    xaxis: { axisBorder: { show: false }, axisTicks: { show: false } },
    yaxis: { tickAmount: 5 },
    markers: createChartMarkers(theme),
    tooltip: { theme: theme.palette.mode, fillSeriesColor: false, x: { show: true } },
    legend: createChartLegend(theme),
    plotOptions: createPlotOptions(theme),
    responsive: createResponsiveOptions(theme),
  };
}

function createChartConfig(theme: Theme): NonNullable<ChartOptions['chart']> {
  return {
    toolbar: { show: false },
    zoom: { enabled: false },
    parentHeightOffset: 0,
    fontFamily: theme.typography.fontFamily,
    foreColor: theme.vars.palette.text.disabled,
    animations: {
      enabled: true,
      speed: 360,
      animateGradually: { enabled: true, delay: 120 },
      dynamicAnimation: { enabled: true, speed: 360 },
    },
  };
}

function createChartColors(theme: Theme): NonNullable<ChartOptions['colors']> {
  return [
    theme.palette.primary.main,
    theme.palette.warning.main,
    theme.palette.info.main,
    theme.palette.error.main,
    theme.palette.success.main,
    theme.palette.warning.dark,
    theme.palette.success.darker,
    theme.palette.info.dark,
    theme.palette.info.darker,
  ];
}

function createChartStates(): NonNullable<ChartOptions['states']> {
  return {
    hover: { filter: { type: 'darken' } },
    active: { filter: { type: 'darken' } },
  };
}

function createChartFill(): NonNullable<ChartOptions['fill']> {
  return {
    opacity: 1,
    gradient: {
      type: 'vertical',
      shadeIntensity: 0,
      opacityFrom: 0.4,
      opacityTo: 0,
      stops: [0, 100],
    },
  };
}

function createChartGrid(theme: Theme): NonNullable<ChartOptions['grid']> {
  return {
    strokeDashArray: 3,
    borderColor: theme.vars.palette.divider,
    padding: { top: 0, right: 0, bottom: 0 },
    xaxis: { lines: { show: false } },
  };
}

function createChartMarkers(theme: Theme): NonNullable<ChartOptions['markers']> {
  return { size: 0, strokeColors: theme.vars.palette.background.paper };
}

function createChartLegend(theme: Theme): NonNullable<ChartOptions['legend']> {
  return {
    show: false,
    position: 'top',
    fontWeight: 500,
    fontSize: '13px',
    horizontalAlign: 'right',
    markers: { shape: 'circle' },
    labels: { colors: theme.vars.palette.text.primary },
    itemMargin: { horizontal: 8, vertical: 8 },
  };
}

function createPlotOptions(theme: Theme): NonNullable<ChartOptions['plotOptions']> {
  return {
    bar: { borderRadius: 4, columnWidth: '48%', borderRadiusApplication: 'end' },
    pie: createPieOptions(theme),
    radialBar: createRadialBarOptions(theme),
    radar: createRadarOptions(theme),
    polarArea: createPolarAreaOptions(theme),
    heatmap: { distributed: true },
  };
}

function createPieOptions(theme: Theme) {
  return {
    donut: {
      labels: {
        show: true,
        value: createValueLabel(theme),
        total: createTotalLabel(theme),
      },
    },
  };
}

function createRadialBarOptions(theme: Theme) {
  return {
    hollow: { margin: -8, size: '100%' },
    track: {
      margin: -8,
      strokeWidth: '50%',
      background: varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
    },
    dataLabels: { value: createValueLabel(theme), total: createTotalLabel(theme) },
  };
}

function createRadarOptions(theme: Theme) {
  return {
    polygons: {
      fill: { colors: ['transparent'] },
      strokeColors: theme.vars.palette.divider,
      connectorColors: theme.vars.palette.divider,
    },
  };
}

function createPolarAreaOptions(theme: Theme) {
  return {
    rings: { strokeColor: theme.vars.palette.divider },
    spokes: { connectorColors: theme.vars.palette.divider },
  };
}

function createTotalLabel(theme: Theme) {
  return {
    show: true,
    label: 'Total',
    color: theme.vars.palette.text.secondary,
    fontSize: theme.typography.subtitle2.fontSize as string,
    fontWeight: theme.typography.subtitle2.fontWeight,
  };
}

function createValueLabel(theme: Theme) {
  return {
    offsetY: 8,
    color: theme.vars.palette.text.primary,
    fontSize: theme.typography.h4.fontSize as string,
    fontWeight: theme.typography.h4.fontWeight,
  };
}

function createResponsiveOptions(theme: Theme): NonNullable<ChartOptions['responsive']> {
  return [
    {
      breakpoint: theme.breakpoints.values.sm,
      options: { plotOptions: { bar: { borderRadius: 3, columnWidth: '80%' } } },
    },
    {
      breakpoint: theme.breakpoints.values.md,
      options: { plotOptions: { bar: { columnWidth: '60%' } } },
    },
  ];
}
