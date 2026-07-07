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

const CAPTCHA_REQUIRED_MESSAGE = 'Complete CAPTCHA first';
const CAPTCHA_LOADING_MESSAGE = 'Captcha config is loading';
const REGISTRATION_LOADING_MESSAGE = 'Registration config is loading';
const REGISTRATION_DISABLED_MESSAGE = 'Registration is disabled';

export type SignInSchemaType = z.infer<typeof SignInSchema>;

export const SignInSchema = z.object({
  identifier: identifierSchema,
  password: passwordSchema,
});

export function JwtSignInView() {
  const controller = useSignInController();

  return (
    <>
      <SiteDocumentTitle title={controller.documentTitle} />
      <SignInHead onGetStarted={controller.handleGetStarted} />
      <AuthErrorAlert message={controller.errorMessage} />
      <Form methods={controller.methods} onSubmit={controller.onSubmit}>
        <SignInForm controller={controller} />
      </Form>
    </>
  );
}

function useSignInController() {
  const router = useRouter();
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const { checkUserSession } = useAuthContext();
  const { data: publicConfigs, isLoading: loadingConfig } = usePublicConfigs();
  const captcha = useCaptchaConfig();
  const captchaState = useCaptchaTokenState();
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const methods = useForm({
    resolver: zodResolver(SignInSchema),
    defaultValues: { identifier: '', password: '' },
  });
  const onSubmit = useSignInSubmit({ methods, captcha, captchaState, setErrorMessage, checkUserSession, router });
  const handleGetStarted = useGetStartedHandler({ router, publicConfigs, loadingConfig });
  const documentTitle = formatPageDocumentTitle('Sign in', siteName);

  return { methods, onSubmit, showPassword, captcha, captchaState, errorMessage, handleGetStarted, documentTitle };
}

type CaptchaTokenState = ReturnType<typeof useCaptchaTokenState>;

function useCaptchaTokenState() {
  const [token, setToken] = useState<string | null>(null);
  const [resetKey, setResetKey] = useState(0);
  const reset = () => {
    setToken(null);
    setResetKey((value) => value + 1);
  };

  return { token, setToken, resetKey, reset };
}

type SignInSubmitOptions = {
  methods: ReturnType<typeof useForm<SignInSchemaType>>;
  captcha: ReturnType<typeof useCaptchaConfig>;
  captchaState: CaptchaTokenState;
  setErrorMessage: (message: string) => void;
  checkUserSession?: () => Promise<void>;
  router: ReturnType<typeof useRouter>;
};

function useSignInSubmit(options: SignInSubmitOptions) {
  const { methods, captcha, captchaState, setErrorMessage, checkUserSession, router } = options;

  return methods.handleSubmit(async (data) => {
    if (!validateCaptcha(captcha, captchaState.token, setErrorMessage)) return;
    try {
      await signInWithPassword({
        identifier: data.identifier,
        password: data.password,
        captchaToken: captcha.data?.enabled ? (captchaState.token ?? undefined) : undefined,
      });
      await checkUserSession?.();
      router.refresh();
    } catch (error) {
      console.error(error);
      setErrorMessage(getErrorMessage(error));
      if (captcha.data?.enabled) captchaState.reset();
    }
  });
}

type GetStartedOptions = {
  router: ReturnType<typeof useRouter>;
  publicConfigs: ReturnType<typeof usePublicConfigs>['data'];
  loadingConfig: boolean;
};

function useGetStartedHandler({ router, publicConfigs, loadingConfig }: GetStartedOptions) {
  return () => {
    if (loadingConfig) {
      toast.info(REGISTRATION_LOADING_MESSAGE);
      return;
    }
    if (!isRegisterEnabled(publicConfigs)) {
      toast.warning(REGISTRATION_DISABLED_MESSAGE);
      return;
    }
    router.push(paths.auth.jwt.signUp);
  };
}

type SignInController = ReturnType<typeof useSignInController>;

function SignInHead({ onGetStarted }: { onGetStarted: () => void }) {
  return (
    <FormHead
      title="Sign in to your account"
      description={
        <>
          {`Don’t have an account? `}
          <Link component="button" type="button" variant="subtitle2" onClick={onGetStarted}>
            Get started
          </Link>
        </>
      }
      sx={{ textAlign: { xs: 'center', md: 'left' } }}
    />
  );
}

function AuthErrorAlert({ message }: { message: string | null }) {
  if (!message) return null;
  return (
    <Alert severity="error" sx={{ mb: 3 }}>
      {message}
    </Alert>
  );
}

function SignInForm({ controller }: { controller: SignInController }) {
  const { methods, showPassword, captcha, captchaState } = controller;
  const { isSubmitting } = methods.formState;

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text name="identifier" label="Username or email" placeholder="username or name@example.com" slotProps={{ inputLabel: { shrink: true } }} />
      <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
        <ForgotPasswordLink />
        <PasswordField show={showPassword.value} onToggle={showPassword.onToggle} />
      </Box>
      <CaptchaWidget config={captcha.data} resetKey={captchaState.resetKey} onTokenChange={captchaState.setToken} />
      {captcha.error ? <Alert severity="error">{getErrorMessage(captcha.error)}</Alert> : null}
      <Button fullWidth color="inherit" size="large" type="submit" variant="contained" loading={isSubmitting || captcha.isLoading} disabled={!!captcha.error} loadingIndicator={captcha.isLoading ? 'Loading captcha...' : 'Sign in...'}>
        Sign in
      </Button>
    </Box>
  );
}

function ForgotPasswordLink() {
  return (
    <Link component={RouterLink} href="#" variant="body2" color="inherit" sx={{ alignSelf: 'flex-end' }}>
      Forgot password?
    </Link>
  );
}

function PasswordField({ show, onToggle }: { show: boolean; onToggle: () => void }) {
  return (
    <Field.Text
      name="password"
      label="Password"
      placeholder="8+ characters"
      type={show ? 'text' : 'password'}
      slotProps={{ inputLabel: { shrink: true }, input: { endAdornment: <PasswordAdornment show={show} onToggle={onToggle} /> } }}
    />
  );
}

function PasswordAdornment({ show, onToggle }: { show: boolean; onToggle: () => void }) {
  return (
    <InputAdornment position="end">
      <IconButton onClick={onToggle} edge="end">
        <Iconify icon={show ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
      </IconButton>
    </InputAdornment>
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
