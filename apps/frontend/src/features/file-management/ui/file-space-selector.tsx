'use client';

import type { AutocompleteRenderInputParams } from '@mui/material/Autocomplete';
import type { FileSpace } from 'src/entities/file';
import type { FileSpaceSelectorState } from '../model/use-file-space-selector';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import Autocomplete from '@mui/material/Autocomplete';
import CircularProgress from '@mui/material/CircularProgress';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

type FileSpaceSelectorProps = Readonly<{
  selector: FileSpaceSelectorState;
  currentUserId: string | undefined;
  label: string;
  onChange: (spaceId: string | undefined) => void;
}>;

type FileSpaceOption =
  | Readonly<{ kind: 'self'; id: '__self__'; label: string }>
  | Readonly<{ kind: 'space'; id: string; label: string; space: FileSpace }>;

export function FileSpaceSelector({
  selector,
  currentUserId,
  label,
  onChange,
}: FileSpaceSelectorProps) {
  const { t } = useTranslate('admin');
  const { t: common } = useTranslate('common');
  const options = useMemo(
    () =>
      fileSpaceOptions(
        selector.selectedSpace,
        selector.spaces.items,
        currentUserId,
        t('file.selfSpace')
      ),
    [currentUserId, selector.selectedSpace, selector.spaces.items, t]
  );
  const value = selectedFileSpaceOption(options, selector.selectedSpace, selector.selectedSpaceId);
  return (
    <Stack
      direction="row"
      spacing={0.5}
      alignItems="center"
      sx={{ minWidth: 240, width: { xs: '100%', md: 'auto' } }}
    >
      <Box sx={{ flex: 1, minWidth: 0, width: { md: 240 } }}>
        <FileSpaceAutocomplete {...{ selector, options, value, label, onChange, t }} />
      </Box>
      <SpacePageControls
        selector={selector}
        previousLabel={common('pagination.previous')}
        nextLabel={common('pagination.next')}
      />
    </Stack>
  );
}

function FileSpaceAutocomplete({
  selector,
  options,
  value,
  label,
  onChange,
  t,
}: Readonly<{
  selector: FileSpaceSelectorState;
  options: readonly FileSpaceOption[];
  value: FileSpaceOption | undefined;
  label: string;
  onChange: (spaceId: string | undefined) => void;
  t: ReturnType<typeof useTranslate>['t'];
}>) {
  return (
    <Autocomplete
      disableClearable
      size="small"
      options={options}
      value={value}
      loading={selector.spaces.isValidating}
      filterOptions={(items) => items}
      isOptionEqualToValue={(option, selected) => option.id === selected.id}
      getOptionLabel={(option) => option.label}
      noOptionsText={t('file.noSpace')}
      onChange={(_, option) => selectFileSpace({ option, selector, onChange })}
      onInputChange={(_, inputValue, reason) => {
        if (reason === 'input') selector.setSearch(inputValue);
      }}
      renderInput={(params) => (
        <FileSpaceInput params={params} selector={selector} label={label} t={t} />
      )}
    />
  );
}

function FileSpaceInput({
  params,
  selector,
  label,
  t,
}: Readonly<{
  params: AutocompleteRenderInputParams;
  selector: FileSpaceSelectorState;
  label: string;
  t: ReturnType<typeof useTranslate>['t'];
}>) {
  return (
    <TextField
      {...params}
      label={label}
      error={Boolean(selector.spaces.error)}
      helperText={selector.spaces.error ? t('file.messages.spacesFailed') : undefined}
      slotProps={{
        input: {
          ...params.InputProps,
          endAdornment: (
            <>
              {selector.spaces.isValidating ? <CircularProgress size={16} /> : null}
              {params.InputProps.endAdornment}
            </>
          ),
        },
      }}
    />
  );
}

function SpacePageControls({
  selector,
  previousLabel,
  nextLabel,
}: Readonly<{
  selector: FileSpaceSelectorState;
  previousLabel: string;
  nextLabel: string;
}>) {
  const { spaces, table } = selector;
  return (
    <Box sx={{ display: 'flex', flexShrink: 0, gap: 0.25 }}>
      <IconButton
        size="small"
        aria-label={previousLabel}
        disabled={spaces.isValidating || !spaces.hasPrevious}
        onClick={() => table.onPreviousCursor(spaces.previousCursor)}
      >
        <Iconify icon="eva:arrow-ios-back-fill" width={16} />
      </IconButton>
      <IconButton
        size="small"
        aria-label={nextLabel}
        disabled={spaces.isValidating || !spaces.hasNext}
        onClick={() => table.onNextCursor(spaces.nextCursor)}
      >
        <Iconify icon="eva:arrow-ios-forward-fill" width={16} />
      </IconButton>
    </Box>
  );
}

function selectFileSpace({
  option,
  selector,
  onChange,
}: Readonly<{
  option: FileSpaceOption | null;
  selector: FileSpaceSelectorState;
  onChange: (spaceId: string | undefined) => void;
}>) {
  if (!option || option.kind === 'self') {
    selector.rememberSpace(null);
    onChange(undefined);
    return;
  }
  selector.rememberSpace(option.space);
  onChange(option.space.id);
}

function fileSpaceOptions(
  selectedSpace: FileSpace | null,
  spaces: readonly FileSpace[],
  currentUserId: string | undefined,
  selfLabel: string
): readonly FileSpaceOption[] {
  const visible = spaces.filter((space) => space.owner_user_id !== currentUserId);
  const selected =
    selectedSpace && !visible.some((space) => space.id === selectedSpace.id)
      ? [selectedSpace, ...visible]
      : visible;
  return [{ kind: 'self', id: '__self__', label: selfLabel }, ...selected.map(toFileSpaceOption)];
}

function selectedFileSpaceOption(
  options: readonly FileSpaceOption[],
  selectedSpace: FileSpace | null,
  selectedSpaceId: string | undefined
): FileSpaceOption | undefined {
  if (selectedSpaceId && !selectedSpace) return undefined;
  if (!selectedSpace) return options[0];
  return options.find((option) => option.id === selectedSpace.id);
}

function toFileSpaceOption(space: FileSpace): FileSpaceOption {
  return {
    kind: 'space',
    id: space.id,
    label: space.department_name
      ? `${space.owner_name} (${space.department_name})`
      : space.owner_name,
    space,
  };
}
