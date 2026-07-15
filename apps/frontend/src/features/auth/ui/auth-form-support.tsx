'use client';

import type { TranslateFn } from 'src/shared/i18n';
import type { useCaptchaConfig } from 'src/features/auth/captcha';

import { useState } from 'react';

import Alert from '@mui/material/Alert';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { Iconify } from 'src/shared/ui/iconify';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { isCaptchaReady } from 'src/features/auth/captcha';

export function useCaptchaTokenState() {
  const [token, setToken] = useState<string | null>(null);
  const [resetKey, setResetKey] = useState(0);
  const reset = () => {
    setToken(null);
    setResetKey((value) => value + 1);
  };

  return { token, setToken, resetKey, reset };
}

type CaptchaValidationOptions = {
  captcha: ReturnType<typeof useCaptchaConfig>;
  token: string | null;
  setErrorMessage: (message: string) => void;
  t: TranslateFn;
};

export function validateCaptcha(options: CaptchaValidationOptions) {
  if (options.captcha.error) {
    options.setErrorMessage(getErrorMessage(options.captcha.error));
    return false;
  }
  if (options.captcha.isLoading) {
    options.setErrorMessage(options.t('auth.captcha.loading'));
    return false;
  }
  if (!isCaptchaReady(options.captcha.data, options.token)) {
    options.setErrorMessage(options.t('auth.captcha.required'));
    return false;
  }
  return true;
}

export function validationMessages(t: TranslateFn) {
  return {
    usernameLength: (min: number, max: number) => t('auth.validation.usernameLength', { min, max }),
    usernamePattern: t('auth.validation.usernamePattern'),
    passwordLength: (min: number, max: number) => t('auth.validation.passwordLength', { min, max }),
    passwordLetterRequired: t('auth.validation.passwordLetterRequired'),
    passwordNumberRequired: t('auth.validation.passwordNumberRequired'),
    passwordSymbolRequired: t('auth.validation.passwordSymbolRequired'),
    passwordContainsUsername: t('auth.validation.passwordContainsUsername'),
    emailRequired: t('auth.validation.emailRequired'),
    emailInvalid: t('auth.validation.emailInvalid'),
    identifierRequired: t('auth.validation.identifierRequired'),
    identifierInvalid: t('auth.validation.identifierInvalid'),
  };
}

export function AuthErrorAlert({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

export function PasswordVisibilityAdornment({
  show,
  onToggle,
}: {
  show: boolean;
  onToggle: () => void;
}) {
  return (
    <InputAdornment position="end">
      <IconButton onClick={onToggle} edge="end">
        <Iconify icon={show ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
      </IconButton>
    </InputAdornment>
  );
}
