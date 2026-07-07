'use client';

import * as z from 'zod';
import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import InputAdornment from '@mui/material/InputAdornment';

import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';
import { useRouter } from 'src/shared/routes/hooks';
import { Form, Field } from 'src/shared/ui/hook-form';
import { RouterLink } from 'src/shared/routes/components';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import {
  emailSchema,
  useAuthContext,
  usernameSchema,
  createPasswordSchema,
} from 'src/entities/session';
import {
  usePublicConfigs,
  isRegisterEnabled,
  passwordPolicyFromPublicConfigs,
} from 'src/entities/system';

import { signUp } from 'src/features/auth';
import { FormHead } from 'src/features/auth/ui/form-head';
import { SignUpTerms } from 'src/features/auth/ui/sign-up-terms';
import { CaptchaWidget, isCaptchaReady, useCaptchaConfig } from 'src/features/auth/captcha';

const CAPTCHA_REQUIRED_MESSAGE = 'Complete CAPTCHA first';
const CAPTCHA_LOADING_MESSAGE = 'Captcha config is loading';
const PASSWORD_CONTAINS_USERNAME_MESSAGE = 'Password cannot contain username';

const DefaultSignUpSchema = z.object({
  username: usernameSchema,
  email: emailSchema,
  password: createPasswordSchema(),
});

export type SignUpSchemaType = z.infer<typeof DefaultSignUpSchema>;

export function JwtSignUpView() {
  const controller = useSignUpController();

  return (
    <>
      <SiteDocumentTitle title={controller.documentTitle} />
      <SignUpContent controller={controller} />
    </>
  );
}

function useSignUpController() {
  const router = useRouter();
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const { checkUserSession } = useAuthContext();
  const config = usePublicConfigs();
  const captcha = useCaptchaConfig();
  const captchaState = useCaptchaTokenState();
  const passwordPolicy = useMemo(() => passwordPolicyFromPublicConfigs(config.data), [config.data]);
  const signUpSchema = useMemo(() => createSignUpSchema(passwordPolicy), [passwordPolicy]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const methods = useForm({
    resolver: zodResolver(signUpSchema),
    defaultValues: { username: '', email: '', password: '' },
  });
  const onSubmit = useSignUpSubmit({ methods, captcha, captchaState, setErrorMessage, checkUserSession, router });
  const documentTitle = formatPageDocumentTitle('Sign up', siteName);

  return { methods, onSubmit, showPassword, config, captcha, captchaState, errorMessage, documentTitle };
}

function createSignUpSchema(policy: ReturnType<typeof passwordPolicyFromPublicConfigs>) {
  const password = createPasswordSchema(policy);
  return DefaultSignUpSchema.extend({ password }).superRefine((data, context) => {
    if (!passwordContainsUsername(data.password, data.username, policy)) return;
    context.addIssue({ code: 'custom', path: ['password'], message: PASSWORD_CONTAINS_USERNAME_MESSAGE });
  });
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

type SignUpSubmitOptions = {
  methods: ReturnType<typeof useForm<SignUpSchemaType>>;
  captcha: ReturnType<typeof useCaptchaConfig>;
  captchaState: CaptchaTokenState;
  setErrorMessage: (message: string) => void;
  checkUserSession?: () => Promise<void>;
  router: ReturnType<typeof useRouter>;
};

function useSignUpSubmit(options: SignUpSubmitOptions) {
  const { methods, captcha, captchaState, setErrorMessage, checkUserSession, router } = options;

  return methods.handleSubmit(async (data) => {
    if (!validateCaptcha(captcha, captchaState.token, setErrorMessage)) return;
    try {
      await signUp({
        username: data.username,
        email: data.email,
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

type SignUpController = ReturnType<typeof useSignUpController>;

function SignUpContent({ controller }: { controller: SignUpController }) {
  const { config, captcha } = controller;

  if (config.isLoading || captcha.isLoading) return <Alert severity="info">Loading registration config...</Alert>;
  if (config.error || captcha.error) return <Alert severity="error">{getErrorMessage(config.error ?? captcha.error)}</Alert>;
  if (!isRegisterEnabled(config.data)) return <RegistrationDisabled />;
  return <RegistrationForm controller={controller} />;
}

function RegistrationDisabled() {
  return (
    <>
      <FormHead
        title="Registration is disabled"
        description="Account self-registration is currently closed."
        sx={{ textAlign: { xs: 'center', md: 'left' } }}
      />
      <Button component={RouterLink} href={paths.auth.jwt.signIn} fullWidth variant="contained" color="inherit">
        Back to sign in
      </Button>
    </>
  );
}

function RegistrationForm({ controller }: { controller: SignUpController }) {
  return (
    <>
      <SignUpHead />
      <AuthErrorAlert message={controller.errorMessage} />
      <Form methods={controller.methods} onSubmit={controller.onSubmit}>
        <SignUpFormFields controller={controller} />
      </Form>
      <SignUpTerms />
    </>
  );
}

function SignUpHead() {
  return (
    <FormHead
      title="Get started absolutely free"
      description={
        <>
          {`Already have an account? `}
          <Link component={RouterLink} href={paths.auth.jwt.signIn} variant="subtitle2">
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

function SignUpFormFields({ controller }: { controller: SignUpController }) {
  const { methods, showPassword, captcha, captchaState } = controller;
  const { isSubmitting } = methods.formState;

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text name="username" label="Username" placeholder="username" slotProps={{ inputLabel: { shrink: true } }} />
      <Field.Text name="email" label="Email address" placeholder="name@example.com" slotProps={{ inputLabel: { shrink: true } }} />
      <PasswordField show={showPassword.value} onToggle={showPassword.onToggle} />
      <CaptchaWidget config={captcha.data} resetKey={captchaState.resetKey} onTokenChange={captchaState.setToken} />
      <Button fullWidth color="inherit" size="large" type="submit" variant="contained" loading={isSubmitting} loadingIndicator="Create account...">
        Create account
      </Button>
    </Box>
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

function passwordContainsUsername(
  password: string,
  username: string,
  policy: ReturnType<typeof passwordPolicyFromPublicConfigs>
) {
  if (!policy?.forbid_username_contains) return false;
  const normalizedUsername = username.trim().toLowerCase();
  return !!normalizedUsername && password.toLowerCase().includes(normalizedUsername);
}
