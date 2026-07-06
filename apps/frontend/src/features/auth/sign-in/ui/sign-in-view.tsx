'use client';

import * as z from 'zod';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { toast } from 'src/shared/ui/snackbar';
import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';
import { useRouter } from 'src/shared/routes/hooks';
import { Form, Field } from 'src/shared/ui/hook-form';
import { RouterLink } from 'src/shared/routes/components';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { usePublicConfigs, isRegisterEnabled } from 'src/entities/system';
import { useAuthContext, passwordSchema, identifierSchema } from 'src/entities/session';

import { signInWithPassword } from 'src/features/auth';
import { FormHead } from 'src/features/auth/ui/form-head';
import { CaptchaWidget, isCaptchaReady, useCaptchaConfig } from 'src/features/auth/captcha';

// ----------------------------------------------------------------------

const CAPTCHA_REQUIRED_MESSAGE = 'Complete CAPTCHA first';
const CAPTCHA_LOADING_MESSAGE = 'Captcha config is loading';

export type SignInSchemaType = z.infer<typeof SignInSchema>;

export const SignInSchema = z.object({
  identifier: identifierSchema,
  password: passwordSchema,
});

// ----------------------------------------------------------------------

export function JwtSignInView() {
  const router = useRouter();
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const { checkUserSession } = useAuthContext();
  const { data: publicConfigs, isLoading: loadingConfig } = usePublicConfigs();
  const captcha = useCaptchaConfig();

  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const methods = useForm({
    resolver: zodResolver(SignInSchema),
    defaultValues: { identifier: '', password: '' },
  });

  const {
    handleSubmit,
    formState: { isSubmitting },
  } = methods;

  const resetCaptcha = () => {
    setCaptchaToken(null);
    setCaptchaResetKey((value) => value + 1);
  };

  const onSubmit = handleSubmit(async (data) => {
    if (!validateCaptcha(captcha, captchaToken, setErrorMessage)) {
      return;
    }

    try {
      await signInWithPassword({
        identifier: data.identifier,
        password: data.password,
        captchaToken: captcha.data?.enabled ? (captchaToken ?? undefined) : undefined,
      });
      await checkUserSession?.();
      router.refresh();
    } catch (error) {
      console.error(error);
      setErrorMessage(getErrorMessage(error));
      if (captcha.data?.enabled) resetCaptcha();
    }
  });

  const handleGetStarted = () => {
    if (loadingConfig) {
      toast.info('Registration config is loading');
      return;
    }
    if (!isRegisterEnabled(publicConfigs)) {
      toast.warning('Registration is disabled');
      return;
    }
    router.push(paths.auth.jwt.signUp);
  };

  const renderForm = () => (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="identifier"
        label="Username or email"
        placeholder="username or name@example.com"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
        <Link
          component={RouterLink}
          href="#"
          variant="body2"
          color="inherit"
          sx={{ alignSelf: 'flex-end' }}
        >
          Forgot password?
        </Link>

        <Field.Text
          name="password"
          label="Password"
          placeholder="8+ characters"
          type={showPassword.value ? 'text' : 'password'}
          slotProps={{
            inputLabel: { shrink: true },
            input: {
              endAdornment: (
                <InputAdornment position="end">
                  <IconButton onClick={showPassword.onToggle} edge="end">
                    <Iconify
                      icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'}
                    />
                  </IconButton>
                </InputAdornment>
              ),
            },
          }}
        />
      </Box>

      <CaptchaWidget
        config={captcha.data}
        resetKey={captchaResetKey}
        onTokenChange={setCaptchaToken}
      />

      {captcha.error ? <Alert severity="error">{getErrorMessage(captcha.error)}</Alert> : null}

      <Button
        fullWidth
        color="inherit"
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting || captcha.isLoading}
        disabled={!!captcha.error}
        loadingIndicator={captcha.isLoading ? 'Loading captcha...' : 'Sign in...'}
      >
        Sign in
      </Button>
    </Box>
  );

  return (
    <>
      <SiteDocumentTitle title={formatPageDocumentTitle('Sign in', siteName)} />

      <FormHead
        title="Sign in to your account"
        description={
          <>
            {`Don’t have an account? `}
            <Link component="button" type="button" variant="subtitle2" onClick={handleGetStarted}>
              Get started
            </Link>
          </>
        }
        sx={{ textAlign: { xs: 'center', md: 'left' } }}
      />

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {errorMessage}
        </Alert>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>
    </>
  );
}

function validateCaptcha(
  captcha: ReturnType<typeof useCaptchaConfig>,
  token: string | null,
  setErrorMessage: (message: string) => void
) {
  if (captcha.error) {
    setErrorMessage(getErrorMessage(captcha.error));
    return false;
  }
  if (captcha.isLoading) {
    setErrorMessage(CAPTCHA_LOADING_MESSAGE);
    return false;
  }
  if (!isCaptchaReady(captcha.data, token)) {
    setErrorMessage(CAPTCHA_REQUIRED_MESSAGE);
    return false;
  }
  return true;
}
