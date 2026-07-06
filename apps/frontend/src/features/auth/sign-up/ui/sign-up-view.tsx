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

// ----------------------------------------------------------------------

const CAPTCHA_REQUIRED_MESSAGE = 'Complete CAPTCHA first';
const CAPTCHA_LOADING_MESSAGE = 'Captcha config is loading';

const DefaultSignUpSchema = z.object({
  username: usernameSchema,
  email: emailSchema,
  password: createPasswordSchema(),
});

export type SignUpSchemaType = z.infer<typeof DefaultSignUpSchema>;

// ----------------------------------------------------------------------

export function JwtSignUpView() {
  const router = useRouter();
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const documentTitle = formatPageDocumentTitle('Sign up', siteName);
  const { checkUserSession } = useAuthContext();
  const { data: publicConfigs, error: configError, isLoading: loadingConfig } = usePublicConfigs();
  const captcha = useCaptchaConfig();
  const passwordPolicy = useMemo(
    () => passwordPolicyFromPublicConfigs(publicConfigs),
    [publicConfigs]
  );
  const signUpSchema = useMemo(() => {
    const passwordSchema = createPasswordSchema(passwordPolicy);
    return DefaultSignUpSchema.extend({ password: passwordSchema }).superRefine((data, context) => {
      if (!passwordContainsUsername(data.password, data.username, passwordPolicy)) {
        return;
      }
      context.addIssue({
        code: 'custom',
        path: ['password'],
        message: 'Password cannot contain username',
      });
    });
  }, [passwordPolicy]);

  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const [captchaResetKey, setCaptchaResetKey] = useState(0);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const methods = useForm({
    resolver: zodResolver(signUpSchema),
    defaultValues: { username: '', email: '', password: '' },
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
      await signUp({
        username: data.username,
        email: data.email,
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

  const renderForm = () => (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="username"
        label="Username"
        placeholder="username"
        slotProps={{ inputLabel: { shrink: true } }}
      />

      <Field.Text
        name="email"
        label="Email address"
        placeholder="name@example.com"
        slotProps={{ inputLabel: { shrink: true } }}
      />

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
                  <Iconify icon={showPassword.value ? 'solar:eye-bold' : 'solar:eye-closed-bold'} />
                </IconButton>
              </InputAdornment>
            ),
          },
        }}
      />

      <CaptchaWidget
        config={captcha.data}
        resetKey={captchaResetKey}
        onTokenChange={setCaptchaToken}
      />

      <Button
        fullWidth
        color="inherit"
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator="Create account..."
      >
        Create account
      </Button>
    </Box>
  );

  if (loadingConfig || captcha.isLoading) {
    return (
      <>
        <SiteDocumentTitle title={documentTitle} />
        <Alert severity="info">Loading registration config...</Alert>
      </>
    );
  }

  if (configError || captcha.error) {
    return (
      <>
        <SiteDocumentTitle title={documentTitle} />
        <Alert severity="error">{getErrorMessage(configError ?? captcha.error)}</Alert>
      </>
    );
  }

  if (!isRegisterEnabled(publicConfigs)) {
    return (
      <>
        <SiteDocumentTitle title={documentTitle} />

        <FormHead
          title="Registration is disabled"
          description="Account self-registration is currently closed."
          sx={{ textAlign: { xs: 'center', md: 'left' } }}
        />
        <Button
          component={RouterLink}
          href={paths.auth.jwt.signIn}
          fullWidth
          variant="contained"
          color="inherit"
        >
          Back to sign in
        </Button>
      </>
    );
  }

  return (
    <>
      <SiteDocumentTitle title={documentTitle} />

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

      {!!errorMessage && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {errorMessage}
        </Alert>
      )}

      <Form methods={methods} onSubmit={onSubmit}>
        {renderForm()}
      </Form>

      <SignUpTerms />
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

function passwordContainsUsername(
  password: string,
  username: string,
  policy: ReturnType<typeof passwordPolicyFromPublicConfigs>
) {
  if (!policy?.forbid_username_contains) {
    return false;
  }
  const normalizedUsername = username.trim().toLowerCase();
  return !!normalizedUsername && password.toLowerCase().includes(normalizedUsername);
}
