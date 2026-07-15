'use client';

import type { BoxProps } from '@mui/material/Box';
import type { InputBaseProps } from '@mui/material/InputBase';
import type { ButtonBaseProps } from '@mui/material/ButtonBase';
import type { FormHelperTextProps } from '@mui/material/FormHelperText';

import { useId, useCallback } from 'react';
import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';

import { Iconify } from '../iconify';
import {
  HelperText,
  CaptionText,
  CenteredInput,
  CounterButton,
  InputContainer,
  NumberInputRoot,
} from './styles';

// ----------------------------------------------------------------------

type NumberInputSlotProps = {
  wrapper?: BoxProps;
  input?: InputBaseProps;
  button?: ButtonBaseProps;
  inputWrapper?: React.ComponentProps<typeof InputContainer>;
  captionText?: React.ComponentProps<typeof CaptionText>;
  helperText?: FormHelperTextProps;
};

type EventHandler =
  React.MouseEvent<HTMLButtonElement, MouseEvent> | React.ChangeEvent<HTMLInputElement>;

export type NumberInputProps = Omit<React.ComponentProps<typeof NumberInputRoot>, 'onChange'> & {
  min?: number;
  max?: number;
  error?: boolean;
  disabled?: boolean;
  value?: number | null;
  hideDivider?: boolean;
  hideButtons?: boolean;
  disableInput?: boolean;
  helperText?: React.ReactNode;
  captionText?: React.ReactNode;
  slotProps?: NumberInputSlotProps;
  onChange?: (event: EventHandler, value: number) => void;
};

type NumberInputStateOptions = {
  value?: number | null;
  min: number;
  max: number;
  disabled?: boolean;
  onChange?: NumberInputProps['onChange'];
};

type NumberInputControlProps = {
  sx: NumberInputProps['sx'];
  error?: boolean;
  disabled?: boolean;
  hideDivider?: boolean;
  hideButtons?: boolean;
  disableInput?: boolean;
  captionText?: React.ReactNode;
  slotProps?: NumberInputSlotProps;
  uniqueId: string;
  rootProps: React.ComponentProps<typeof NumberInputRoot>;
  state: ReturnType<typeof useNumberInputState>;
};

type NumberInputCounterProps = {
  hidden?: boolean;
  disabled?: boolean;
  onClick: (event: React.MouseEvent<HTMLButtonElement, MouseEvent>) => void;
  buttonProps?: ButtonBaseProps;
  icon: 'mingcute:minimize-line' | 'mingcute:add-line';
};

export function NumberInput({
  sx,
  error,
  value,
  onChange,
  disabled,
  slotProps,
  helperText,
  captionText,
  hideDivider,
  hideButtons,
  disableInput,
  min = 0,
  max = 9999,
  ...other
}: NumberInputProps) {
  const uniqueId = useId();
  const state = useNumberInputState({ value, min, max, disabled, onChange });

  return (
    <Box {...slotProps?.wrapper}>
      <NumberInputControl
        sx={sx}
        error={error}
        state={state}
        disabled={disabled}
        slotProps={slotProps}
        uniqueId={uniqueId}
        hideDivider={hideDivider}
        hideButtons={hideButtons}
        disableInput={disableInput}
        captionText={captionText}
        rootProps={other}
      />

      {helperText && (
        <HelperText error={error} {...slotProps?.helperText}>
          {helperText}
        </HelperText>
      )}
    </Box>
  );
}

function useNumberInputState(options: NumberInputStateOptions) {
  const { value, min, max, disabled, onChange } = options;
  const currentValue = value ?? 0;
  const isDecrementDisabled = currentValue <= min || disabled;
  const isIncrementDisabled = currentValue >= max || disabled;

  const handleDecrement = useCallback(
    (event: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
      if (isDecrementDisabled) return;
      onChange?.(event, currentValue - 1);
    },
    [isDecrementDisabled, onChange, currentValue]
  );

  const handleIncrement = useCallback(
    (event: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
      if (isIncrementDisabled) return;
      onChange?.(event, currentValue + 1);
    },
    [isIncrementDisabled, onChange, currentValue]
  );

  const handleChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const transformedValue = transformNumberOnChange(event.target.value, { min, max });
      onChange?.(event, transformedValue);
    },
    [max, min, onChange]
  );

  return {
    currentValue,
    isDecrementDisabled,
    isIncrementDisabled,
    handleDecrement,
    handleIncrement,
    handleChange,
  };
}

function NumberInputControl(props: NumberInputControlProps) {
  const slots = props.slotProps ?? {};

  return (
    <NumberInputRoot
      sx={[
        (theme) => ({
          '--border-color': varAlpha(theme.vars.palette.grey['500Channel'], 0.2),
          '--vertical-divider-color': props.hideDivider
            ? 'transparent'
            : varAlpha(theme.vars.palette.grey['500Channel'], 0.2),
          '--input-background':
            !props.disabled && props.error
              ? varAlpha(theme.vars.palette.error.mainChannel, 0.08)
              : varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
        }),
        ...(Array.isArray(props.sx) ? props.sx : [props.sx]),
      ]}
      {...props.rootProps}
    >
      <NumberInputCounter
        hidden={props.hideButtons}
        disabled={props.state.isDecrementDisabled}
        onClick={props.state.handleDecrement}
        buttonProps={slots.button}
        icon="mingcute:minimize-line"
      />
      <InputContainer {...slots.inputWrapper}>
        <CenteredInput
          name={props.uniqueId}
          disabled={props.disabled || props.disableInput}
          value={props.state.currentValue}
          onChange={props.state.handleChange}
          {...slots.input}
        />
        {props.captionText && <CaptionText {...slots.captionText}>{props.captionText}</CaptionText>}
      </InputContainer>
      <NumberInputCounter
        hidden={props.hideButtons}
        disabled={props.state.isIncrementDisabled}
        onClick={props.state.handleIncrement}
        buttonProps={slots.button}
        icon="mingcute:add-line"
      />
    </NumberInputRoot>
  );
}

function NumberInputCounter({
  hidden,
  disabled,
  onClick,
  buttonProps,
  icon,
}: NumberInputCounterProps) {
  if (hidden) return null;

  return (
    <CounterButton disabled={disabled} onClick={onClick} {...buttonProps}>
      <Iconify width={16} icon={icon} />
    </CounterButton>
  );
}

// ----------------------------------------------------------------------

export function transformNumberOnChange(
  value: string,
  options?: { min?: number; max?: number }
): number {
  const { min = 0, max = 9999 } = options ?? {};

  if (!value || value.trim() === '') {
    return 0;
  }

  const numericValue = Number(value.trim());

  if (!Number.isNaN(numericValue)) {
    // Clamp the value between min and max
    return Math.min(Math.max(numericValue, min), max);
  }

  return 0;
}
