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

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n';
import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';
import { Form, Field } from 'src/shared/ui/hook-form';
import { RouterLink } from 'src/shared/routes/components';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { usePublicConfigs, isRegisterEnabled } from 'src/entities/system';
import {
  useAuthContext,
  createIdentifierSchema,
  createBasicPasswordSchema,
} from 'src/entities/session';

import { signInWithPassword } from 'src/features/auth';
import { FormHead } from 'src/features/auth/ui/form-head';
import { CaptchaWidget, useCaptchaConfig } from 'src/features/auth/captcha';
import {
  AuthErrorAlert,
  validateCaptcha,
  validationMessages,
  useCaptchaTokenState,
  PasswordVisibilityAdornment,
} from 'src/features/auth/ui/auth-form-support';

export type SignInSchemaType = z.infer<ReturnType<typeof createSignInSchema>>;

export function JwtSignInView() {
  const controller = useSignInController();

  return (
    <>
      <SiteDocumentTitle title={controller.documentTitle} />
      <SignInHead t={controller.t} onGetStarted={controller.handleGetStarted} />
      <AuthErrorAlert message={controller.errorMessage} />
      <Form methods={controller.methods} onSubmit={controller.onSubmit}>
        <SignInForm controller={controller} />
      </Form>
    </>
  );
}

function useSignInController() {
  const router = useRouter();
  const { t } = useTranslate('messages');
  const showPassword = useBoolean();
  const { siteName } = useSiteDisplay();
  const { checkUserSession } = useAuthContext();
  const { data: publicConfigs, isLoading: loadingConfig } = usePublicConfigs();
  const captcha = useCaptchaConfig();
  const captchaState = useCaptchaTokenState();
  const schema = useMemo(() => createSignInSchema(t), [t]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const methods = useForm<SignInSchemaType>({
    resolver: zodResolver(schema),
    defaultValues: { identifier: '', password: '' },
  });
  const onSubmit = useSignInSubmit({
    methods,
    captcha,
    captchaState,
    setErrorMessage,
    checkUserSession,
    router,
    t,
  });
  const handleGetStarted = useGetStartedHandler({ router, publicConfigs, loadingConfig, t });
  const documentTitle = formatPageDocumentTitle(t('auth.signIn.documentTitle'), siteName);

  return {
    methods,
    onSubmit,
    showPassword,
    captcha,
    captchaState,
    errorMessage,
    handleGetStarted,
    documentTitle,
    t,
  };
}

function createSignInSchema(t: TranslateFn) {
  const messages = validationMessages(t);
  return z.object({
    identifier: createIdentifierSchema(messages),
    password: createBasicPasswordSchema(messages),
  });
}

type CaptchaTokenState = ReturnType<typeof useCaptchaTokenState>;

type SignInSubmitOptions = {
  methods: ReturnType<typeof useForm<SignInSchemaType>>;
  captcha: ReturnType<typeof useCaptchaConfig>;
  captchaState: CaptchaTokenState;
  setErrorMessage: (message: string) => void;
  checkUserSession: () => Promise<void>;
  router: ReturnType<typeof useRouter>;
  t: TranslateFn;
};

function useSignInSubmit(options: SignInSubmitOptions) {
  const { methods, captcha, captchaState, setErrorMessage, checkUserSession, router, t } = options;

  return methods.handleSubmit(async (data) => {
    if (!validateCaptcha({ captcha, token: captchaState.token, setErrorMessage, t })) return;
    try {
      await signInWithPassword({
        identifier: data.identifier,
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

type GetStartedOptions = {
  router: ReturnType<typeof useRouter>;
  publicConfigs: ReturnType<typeof usePublicConfigs>['data'];
  loadingConfig: boolean;
  t: TranslateFn;
};

function useGetStartedHandler({ router, publicConfigs, loadingConfig, t }: GetStartedOptions) {
  return () => {
    if (loadingConfig) {
      toast.info(t('auth.signIn.registrationLoading'));
      return;
    }
    if (!isRegisterEnabled(publicConfigs)) {
      toast.warning(t('auth.signIn.registrationDisabled'));
      return;
    }
    router.push(paths.auth.jwt.signUp);
  };
}

type SignInController = ReturnType<typeof useSignInController>;

function SignInHead({ t, onGetStarted }: { t: TranslateFn; onGetStarted: () => void }) {
  return (
    <FormHead
      title={t('auth.signIn.title')}
      description={
        <>
          {`${t('auth.signIn.noAccount')} `}
          <Link component="button" type="button" variant="subtitle2" onClick={onGetStarted}>
            {t('auth.signIn.getStarted')}
          </Link>
        </>
      }
      sx={{ textAlign: { xs: 'center', md: 'left' } }}
    />
  );
}

function SignInForm({ controller }: { controller: SignInController }) {
  const { methods, showPassword, captcha, captchaState, t } = controller;
  const { isSubmitting } = methods.formState;

  return (
    <Box sx={{ gap: 3, display: 'flex', flexDirection: 'column' }}>
      <Field.Text
        name="identifier"
        label={t('auth.signIn.identifierLabel')}
        placeholder={t('auth.signIn.identifierPlaceholder')}
        slotProps={{ inputLabel: { shrink: true } }}
      />
      <Box sx={{ gap: 1.5, display: 'flex', flexDirection: 'column' }}>
        <ForgotPasswordLink t={t} />
        <PasswordField t={t} show={showPassword.value} onToggle={showPassword.onToggle} />
      </Box>
      <CaptchaWidget
        config={captcha.data}
        resetKey={captchaState.resetKey}
        onTokenChange={captchaState.setToken}
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
        loadingIndicator={
          captcha.isLoading ? t('auth.captcha.loading') : t('auth.signIn.submitting')
        }
      >
        {t('auth.signIn.submit')}
      </Button>
    </Box>
  );
}

function ForgotPasswordLink({ t }: { t: TranslateFn }) {
  return (
    <Link
      component={RouterLink}
      href="#"
      variant="body2"
      color="inherit"
      sx={{ alignSelf: 'flex-end' }}
    >
      {t('auth.signIn.forgotPassword')}
    </Link>
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
      label={t('auth.signIn.passwordLabel')}
      placeholder={t('auth.signIn.passwordPlaceholder')}
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
