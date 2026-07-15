'use client';

import type { Theme, SxProps } from '@mui/material/styles';

import Box from '@mui/material/Box';
import Select from '@mui/material/Select';
import Switch from '@mui/material/Switch';
import MenuItem from '@mui/material/MenuItem';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import FormControlLabel from '@mui/material/FormControlLabel';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { CURSOR_LIMIT_OPTIONS } from 'src/shared/api/pagination';
import { isCursorNavigationDisabled } from 'src/shared/lib/use-cursor-navigation';

export type CursorPaginationProps = Readonly<{
  limit: number;
  itemCount: number;
  visitedBatchIndex: number;
  hasPrevious: boolean;
  hasNext: boolean;
  pending: boolean;
  dense?: boolean;
  sx?: SxProps<Theme>;
  limitOptions?: readonly number[];
  onPrevious: () => void;
  onNext: () => void;
  onLimitChange: (limit: number) => void;
  onChangeDense?: (event: React.ChangeEvent<HTMLInputElement>) => void;
}>;

type PaginationTranslator = ReturnType<typeof useTranslate>['t'];

const PAGINATION_LAYOUT: SxProps<Theme> = {
  px: 2,
  py: 1,
  gap: 1.5,
  minHeight: 64,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'flex-end',
  borderTop: (theme) => `solid 1px ${theme.vars.palette.divider}`,
};

export function CursorPagination({
  limit,
  itemCount,
  visitedBatchIndex,
  hasPrevious,
  hasNext,
  pending,
  dense,
  sx,
  limitOptions = CURSOR_LIMIT_OPTIONS,
  onPrevious,
  onNext,
  onLimitChange,
  onChangeDense,
}: CursorPaginationProps) {
  const { t } = useTranslate('common');

  return (
    <Box sx={[PAGINATION_LAYOUT, ...(Array.isArray(sx) ? sx : [sx])]}>
      <DenseControl dense={dense} onChange={onChangeDense} t={t} />
      <LimitControl
        limit={limit}
        pending={pending}
        options={limitOptions}
        onChange={onLimitChange}
        t={t}
      />
      <Typography variant="body2">
        {t('pagination.batchItems', { batch: visitedBatchIndex + 1, count: itemCount })}
      </Typography>
      <NavigationControls
        pending={pending}
        hasPrevious={hasPrevious}
        hasNext={hasNext}
        onPrevious={onPrevious}
        onNext={onNext}
        t={t}
      />
    </Box>
  );
}

type DenseControlProps = Readonly<{
  dense?: boolean;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  t: PaginationTranslator;
}>;

function DenseControl({ dense, onChange, t }: DenseControlProps) {
  if (!onChange) return null;
  return (
    <FormControlLabel
      label={t('pagination.dense')}
      control={<Switch checked={dense} onChange={onChange} />}
      sx={{ mr: 'auto' }}
    />
  );
}

type LimitControlProps = Readonly<{
  limit: number;
  pending: boolean;
  options: readonly number[];
  onChange: (limit: number) => void;
  t: PaginationTranslator;
}>;

function LimitControl({ limit, pending, options, onChange, t }: LimitControlProps) {
  return (
    <>
      <Typography variant="body2">{t('pagination.limit')}</Typography>
      <Select
        size="small"
        value={limit}
        disabled={pending}
        onChange={(event) => onChange(Number(event.target.value))}
        inputProps={{ 'aria-label': t('pagination.limit') }}
      >
        {options.map((option) => (
          <MenuItem key={option} value={option}>
            {option}
          </MenuItem>
        ))}
      </Select>
    </>
  );
}

type NavigationControlsProps = Readonly<{
  pending: boolean;
  hasPrevious: boolean;
  hasNext: boolean;
  onPrevious: () => void;
  onNext: () => void;
  t: PaginationTranslator;
}>;

function NavigationControls({
  pending,
  hasPrevious,
  hasNext,
  onPrevious,
  onNext,
  t,
}: NavigationControlsProps) {
  return (
    <>
      <IconButton
        aria-label={t('pagination.previous')}
        disabled={isCursorNavigationDisabled(pending, hasPrevious)}
        onClick={onPrevious}
      >
        <Iconify icon="eva:arrow-ios-back-fill" />
      </IconButton>
      <IconButton
        aria-label={t('pagination.next')}
        disabled={isCursorNavigationDisabled(pending, hasNext)}
        onClick={onNext}
      >
        <Iconify icon="eva:arrow-ios-forward-fill" />
      </IconButton>
    </>
  );
}
