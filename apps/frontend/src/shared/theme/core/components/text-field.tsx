import type { Theme, Components } from '@mui/material/styles';

import { inputBaseClasses } from '@mui/material/InputBase';
import { filledInputClasses } from '@mui/material/FilledInput';
import { outlinedInputClasses } from '@mui/material/OutlinedInput';

import {
  inputStyles,
  inputBaseStyles,
  filledInputStyles,
  inputBaseVariants,
  filledInputVariants,
  outlinedInputStyles,
  outlinedInputVariants,
  multilineInputVariants,
} from './text-field-common';

export {
  inputStyles,
  INPUT_PADDING,
  inputBaseStyles,
  INPUT_TYPOGRAPHY,
  filledInputStyles,
  inputBaseVariants,
  getInputTypography,
  filledInputVariants,
  outlinedInputStyles,
  outlinedInputVariants,
} from './text-field-common';

// ----------------------------------------------------------------------

const MuiInputBase: Components<Theme>['MuiInputBase'] = {
  styleOverrides: {
    root: ({ theme }) => ({
      ...inputBaseStyles.root('standard', theme, inputBaseClasses),
      variants: inputBaseVariants.root,
    }),
    input: ({ theme }) => ({
      ...inputBaseStyles.input('standard', theme),
      variants: [...inputBaseVariants.input, ...multilineInputVariants],
    }),
  },
};

const MuiInput: Components<Theme>['MuiInput'] = {
  styleOverrides: {
    root: ({ theme }) => inputStyles.root(theme),
  },
};

const MuiOutlinedInput: Components<Theme>['MuiOutlinedInput'] = {
  styleOverrides: {
    root: ({ theme }) => ({
      ...outlinedInputStyles.root(theme, outlinedInputClasses),
      variants: outlinedInputVariants.root,
    }),
    input: { variants: [...outlinedInputVariants.input, ...multilineInputVariants] },
    notchedOutline: ({ theme }) => outlinedInputStyles.notchedOutline(theme),
  },
};

const MuiFilledInput: Components<Theme>['MuiFilledInput'] = {
  defaultProps: {
    disableUnderline: true,
  },
  styleOverrides: {
    root: ({ theme }) => ({
      ...filledInputStyles.root(theme, filledInputClasses),
      variants: filledInputVariants.root,
    }),
    input: {
      variants: [...filledInputVariants.input, ...multilineInputVariants],
    },
  },
};

const MuiTextField: Components<Theme>['MuiTextField'] = {
  defaultProps: {
    variant: 'outlined',
  },
};

export const textField: Components<Theme> = {
  MuiInput,
  MuiInputBase,
  MuiTextField,
  MuiFilledInput,
  MuiOutlinedInput,
};
