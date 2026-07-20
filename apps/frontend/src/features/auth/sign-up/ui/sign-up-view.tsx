'use client';

import type { TranslateFn } from 'src/shared/i18n';

import * as z from 'zod';
import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { useBoolean } from 'minimal-shared/hooks';
import { zodResolver } from '@hookform/resolvers/zod';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n';
import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';
import { Form, Field } from 'src/shared/ui/hook-form';
import { RouterLink } from 'src/shared/routes/components';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import {
  usePublicConfigs,
  isRegisterEnabled,
  passwordPolicyFromPublicConfigs,
} from 'src/entities/system';
import {
  useAuthContext,
  createEmailSchema,
  createUsernameSchema,
  createPasswordSchema,
  passwordContainsUsername,
} from 'src/entities/session';

import { signUp } from 'src/features/auth';
import { FormHead } from 'src/features/auth/ui/form-head';
import { SignUpTerms } from 'src/features/auth/ui/sign-up-terms';
import { CaptchaWidget, useCaptchaConfig } from 'src/features/auth/captcha';
import {
  AuthErrorAlert,
  validateCaptcha,
  validationMessages,
  useCaptchaTokenState,
  PasswordVisibilityAdornment,
} from 'src/features/auth/ui/auth-form-support';

export type SignUpSchemaType = z.infer<ReturnType<typeof createSignUpSchema>>;

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
  const { t } = useTranslate('messages');
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const { checkUserSession } = useAuthContext();
  const config = usePublicConfigs();
  const captcha = useCaptchaConfig();
  const captchaState = useCaptchaTokenState();
  const passwordPolicy = useMemo(() => passwordPolicyFromPublicConfigs(config.data), [config.data]);
  const schema = useMemo(() => createSignUpSchema(t, passwordPolicy), [t, passwordPolicy]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const methods = useForm<SignUpSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: { username: '', email: '', password: '' },
  });
  const onSubmit = useSignUpSubmit({
    methods,
    captcha,
    captchaState,
    setErrorMessage,
    checkUserSession,
    router,
    t,
  });
  const documentTitle = formatPageDocumentTitle(t('auth.signUp.documentTitle'), siteName);

  return {
    methods,
    onSubmit,
    showPassword,
    config,
    captcha,
    captchaState,
    errorMessage,
    documentTitle,
    t,
  };
}

function createSignUpSchema(
  t: TranslateFn,
  policy: ReturnType<typeof passwordPolicyFromPublicConfigs>
) {
  const messages = validationMessages(t);
  return z
    .object({
      username: createUsernameSchema(messages),
      email: createEmailSchema(messages),
      password: createPasswordSchema(messages, policy),
    })
    .superRefine((data, context) => {
      if (!passwordContainsUsername(data.password, data.username, policy)) return;
      context.addIssue({
        code: 'custom',
        path: ['password'],
        message: t('auth.validation.passwordContainsUsername'),
      });
    });
}

type CaptchaTokenState = ReturnType<typeof useCaptchaTokenState>;

type SignUpSubmitOptions = {
  methods: ReturnType<typeof useForm<SignUpSchemaType>>;
  captcha: ReturnType<typeof useCaptchaConfig>;
  captchaState: CaptchaTokenState;
  setErrorMessage: (message: string) => void;
  checkUserSession: () => Promise<void>;
  router: ReturnType<typeof useRouter>;
  t: TranslateFn;
};

function useSignUpSubmit(options: SignUpSubmitOptions) {
  const { methods, captcha, captchaState, setErrorMessage, checkUserSession, router, t } = options;

  return methods.handleSubmit(async (data) => {
    if (!validateCaptcha({ captcha, token: captchaState.token, setErrorMessage, t })) return;
    try {
      await signUp({
        username: data.username,
        email: data.email,
        password: data.password,
        captchaToken: captcha.data?.enabled ? (captchaState.token ?? undefined) : undefined,
      });
      await checkUserSession();
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
  const { config, captcha, t } = controller;

  if (config.isLoading || captcha.isLoading)
    return <Alert severity="info">{t('auth.signUp.loading')}</Alert>;
  if (config.error || captcha.error)
    return <Alert severity="error">{getErrorMessage(config.error ?? captcha.error)}</Alert>;
  if (!isRegisterEnabled(config.data)) return <RegistrationDisabled t={t} />;
  return <RegistrationForm controller={controller} />;
}

function RegistrationDisabled({ t }: { t: TranslateFn }) {
  return (
    <>
      <FormHead
        title={t('auth.signUp.disabledTitle')}
        description={t('auth.signUp.disabledDescription')}
        sx={{ textAlign: { xs: 'center', md: 'left' } }}
      />
      <Button
        component={RouterLink}
        href={paths.auth.jwt.signIn}
        fullWidth
        variant="contained"
        color="inherit"
      >
        {t('auth.signUp.backToSignIn')}
      </Button>
    </>
  );
}

function RegistrationForm({ controller }: { controller: SignUpController }) {
  return (
    <>
      <SignUpHead t={controller.t} />
      <AuthErrorAlert message={controller.errorMessage} />
      <Form methods={controller.methods} onSubmit={controller.onSubmit}>
        <SignUpFormFields controller={controller} />
      </Form>
      <SignUpTerms />
    </>
  );
}

function SignUpHead({ t }: { t: TranslateFn }) {
  return (
    <FormHead
      title={t('auth.signUp.title')}
      description={
        <>
          {`${t('auth.signUp.hasAccount')} `}
          <Link component={RouterLink} href={paths.auth.jwt.signIn} variant="subtitle2">
            {t('auth.signUp.signInLink')}
          </Link>
        </>
      }
      sx={{ textAlign: { xs: 'center', md: 'left' } }}
    />
  );
}

function SignUpFormFields({ controller }: { controller: SignUpController }) {
  const { methods, showPassword, captcha, captchaState, t } = controller;
  const { isSubmitting } = methods.formState;

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="username"
        label={t('auth.signUp.usernameLabel')}
        placeholder={t('auth.signUp.usernamePlaceholder')}
        slotProps={{ inputLabel: { shrink: true } }}
      />
      <Field.Text
        name="email"
        label={t('auth.signUp.emailLabel')}
        placeholder={t('auth.signUp.emailPlaceholder')}
        slotProps={{ inputLabel: { shrink: true } }}
      />
      <PasswordField t={t} show={showPassword.value} onToggle={showPassword.onToggle} />
      <CaptchaWidget
        config={captcha.data}
        resetKey={captchaState.resetKey}
        onTokenChange={captchaState.setToken}
      />
      <Button
        fullWidth
        color="inherit"
        size="large"
        type="submit"
        variant="contained"
        loading={isSubmitting}
        loadingIndicator={t('auth.signUp.submitting')}
      >
        {t('auth.signUp.submit')}
      </Button>
    </Box>
  );
}

function PasswordField({
  t,
  show,
  onToggle,
}: {
  t: TranslateFn;
  show: boolean;
  onToggle: () => void;
}) {
  return (
    <Field.Text
      name="password"
      label={t('auth.signUp.passwordLabel')}
      placeholder={t('auth.signUp.passwordPlaceholder')}
      type={show ? 'text' : 'password'}
      slotProps={{
        inputLabel: { shrink: true },
        input: {
          endAdornment: <PasswordVisibilityAdornment show={show} onToggle={onToggle} />,
        },
      }}
    />
  );
}
