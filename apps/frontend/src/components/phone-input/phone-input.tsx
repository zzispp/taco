import type { TextFieldProps } from '@mui/material/TextField';
import type { PhoneValue, PhoneCountry, PhoneInputProps } from './types';

import { debounce } from 'es-toolkit';
import { useMemo, useState, useCallback } from 'react';
import PhoneNumberInput, { parsePhoneNumber } from 'react-phone-number-input/input';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';
import TextField from '@mui/material/TextField';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';
import { inputBaseClasses } from '@mui/material/InputBase';

import { countries } from 'src/assets/data/countries';

import { Iconify } from '../iconify';
import { CountryListPopover } from './list-popover';

// ----------------------------------------------------------------------

export function PhoneInput({
  sx,
  size,
  label,
  placeholder,
  fullWidth = true,
  variant: variantProp,
  /********/
  value,
  country,
  onChange,
  defaultCountry,
  /********/
  hideSelect,
  ...other
}: PhoneInputProps) {
  const theme = useTheme();
  const variant = variantProp ?? theme.components?.MuiTextField?.defaultProps?.variant;

  const normalizedValue = value ? value.trim().replace(/[\s-]+/g, '') : undefined;

  const [searchCountry, setSearchCountry] = useState('');
  const [selectedCountry, setSelectedCountry] = useState<PhoneCountry | undefined>(
    parseCountryFromPhone(normalizedValue) ?? country ?? defaultCountry
  );

  const hasLabel = !!label;
  const isCountryLocked = !!country;

  const activeCountry = useMemo(() => {
    const parsedCountry = parseCountryFromPhone(normalizedValue);
    return parsedCountry ?? country ?? selectedCountry ?? defaultCountry;
  }, [country, selectedCountry, normalizedValue, defaultCountry]);

  const debouncedChange = useMemo(
    () => debounce((inputValue: PhoneValue) => onChange(inputValue), 200),
    [onChange]
  );

  const handleChangeInput = useCallback(
    (inputValue: PhoneValue) => {
      debouncedChange(inputValue ?? '');
    },
    [debouncedChange]
  );

  const handleClearInput = useCallback(() => {
    handleChangeInput('' as PhoneValue);
  }, [handleChangeInput]);

  const handleSearchCountry = useCallback((inputQuery: string) => {
    setSearchCountry(inputQuery);
  }, []);

  const handleSelectedCountry = useCallback(
    (countryCode: PhoneCountry) => {
      setSearchCountry('');
      handleClearInput();
      setSelectedCountry(countryCode);
    },
    [handleClearInput]
  );

  const renderSelect = () => (
    <CountryListPopover
      options={countries}
      searchCountry={searchCountry}
      selectedCountry={activeCountry}
      onSearchCountry={handleSearchCountry}
      onSelectedCountry={handleSelectedCountry}
      disabled={isCountryLocked}
      sx={{
        pl: variant === 'standard' ? 0 : 1.5,
        ...(variant === 'standard' && hasLabel && { mt: size === 'small' ? '16px' : '20px' }),
        ...((variant === 'filled' || variant === 'outlined') && {
          mt: size === 'small' ? '8px' : '16px',
        }),
        ...(variant === 'filled' && hasLabel && { mt: size === 'small' ? '21px' : '25px' }),
      }}
    />
  );

  const renderInput = () => {
    const textFieldProps: Omit<TextFieldProps, 'value' | 'onChange'> = {
      size,
      label,
      variant,
      fullWidth,
      hiddenLabel: !label,
      placeholder: placeholder ?? 'Enter phone number',
      slotProps: {
        inputLabel: { shrink: true },
        input: {
          endAdornment: normalizedValue && (
            <InputAdornment position="end">
              <IconButton size="small" edge="end" onClick={handleClearInput}>
                <Iconify width={16} icon="mingcute:close-line" />
              </IconButton>
            </InputAdornment>
          ),
        },
      },
    };

    const phoneInputProps: PhoneInputProps = {
      value: normalizedValue,
      onChange: handleChangeInput,
      inputComponent: CustomInput,
      ...(isCountryLocked ? { country: activeCountry } : { defaultCountry: activeCountry }),
    };

    return <PhoneNumberInput {...textFieldProps} {...phoneInputProps} {...other} />;
  };

  const baseButtonWidth = variant === 'standard' ? '48px' : '60px';
  const disabledButtonWidth = `calc(${baseButtonWidth} - 16px)`;
  const buttonWidth = isCountryLocked ? disabledButtonWidth : baseButtonWidth;

  return (
    <Box
      sx={[
        {
          '--popover-button-mr': '12px',
          '--popover-button-height': '22px',
          '--popover-button-width': buttonWidth,
          position: 'relative',
          ...(fullWidth && { width: 1 }),
          ...(!hideSelect && {
            [`& .${inputBaseClasses.input}`]: {
              pl: 'calc(var(--popover-button-width) + var(--popover-button-mr))',
            },
          }),
        },
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      {!hideSelect && renderSelect()}
      {renderInput()}
    </Box>
  );
}

// ----------------------------------------------------------------------

function CustomInput({ ref, ...other }: TextFieldProps) {
  return <TextField inputRef={ref} {...other} />;
}

// ----------------------------------------------------------------------

function parseCountryFromPhone(inputValue?: PhoneInputProps['value']): PhoneCountry | undefined {
  const parsed = inputValue ? parsePhoneNumber(inputValue) : undefined;
  return parsed?.country ?? undefined;
}
