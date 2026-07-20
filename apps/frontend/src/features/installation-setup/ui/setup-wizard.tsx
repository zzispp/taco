'use client';

import type { SetupDefaults } from 'src/entities/installation';

import Box from '@mui/material/Box';
import Step from '@mui/material/Step';
import Paper from '@mui/material/Paper';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Stepper from '@mui/material/Stepper';
import StepLabel from '@mui/material/StepLabel';
import Typography from '@mui/material/Typography';

import { Form } from 'src/shared/ui/hook-form';
import { allLangs, useTranslate } from 'src/shared/i18n';
import { SplashScreen } from 'src/shared/ui/loading-screen';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { LanguagePopover } from 'src/shared/ui/shell/language-popover';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';

import { AdministratorStep } from './administrator-step';
import { InstallationConfirmation } from './confirmation-step';
import { useSetupDefaults } from '../model/use-setup-defaults';
import { RedisConnectionStep, PostgresConnectionStep } from './connection-step';
import { useSetupWizard, type SetupWizardStep } from '../model/use-setup-wizard';

const OPERATOR_STEPS: readonly SetupWizardStep[] = [
  'postgres',
  'redis',
  'administrator',
  'confirmation',
];

export function SetupWizard() {
  const defaults = useSetupDefaults();
  const { t } = useTranslate('setup');

  if (defaults.kind === 'loading') {
    return <SplashScreen />;
  }
  if (defaults.kind === 'failure') {
    return (
      <SetupStateScreen
        title={t('status.defaultsFailedTitle')}
        detail={getErrorMessage(defaults.error)}
        retryLabel={t('actions.retry')}
        onRetry={defaults.retry}
      />
    );
  }

  return <SetupWizardForm defaults={defaults.defaults} />;
}

function SetupWizardForm({ defaults }: Readonly<{ defaults: SetupDefaults }>) {
  const { t: setupT } = useTranslate('setup');
  const { t: messagesT } = useTranslate('messages');
  const controller = useSetupWizard(defaults, setupT, messagesT);
  const title = setupT('documentTitle');

  return (
    <Box sx={{ minHeight: '100vh', p: { xs: 2, md: 4 } }}>
      <SiteDocumentTitle title={title} />
      <Stack alignItems="center" spacing={3}>
        <Stack
          direction="row"
          alignItems="center"
          justifyContent="space-between"
          sx={{ width: 1, maxWidth: 800 }}
        >
          <Typography variant="h5">{setupT('brand')}</Typography>
          <LanguagePopover data={allLangs} />
        </Stack>
        <Paper sx={{ width: 1, maxWidth: 800, p: { xs: 2, md: 4 } }}>
          <Stepper activeStep={stepIndex(controller.step)} alternativeLabel sx={{ mb: 4 }}>
            {OPERATOR_STEPS.map((step) => (
              <Step key={step}>
                <StepLabel>{setupT(`steps.${step}.title`)}</StepLabel>
              </Step>
            ))}
          </Stepper>
          <Form methods={controller.methods}>
            <WizardStepContent controller={controller} />
          </Form>
        </Paper>
      </Stack>
    </Box>
  );
}

type WizardStepContentProps = Readonly<{
  controller: ReturnType<typeof useSetupWizard>;
}>;

function WizardStepContent({ controller }: WizardStepContentProps) {
  if (controller.step === 'postgres') {
    return (
      <PostgresConnectionStep
        onNext={controller.next}
        onTest={controller.connections.testPostgres}
        testing={controller.connections.testingPostgres}
      />
    );
  }
  if (controller.step === 'redis') {
    return (
      <RedisConnectionStep
        onBack={controller.back}
        onNext={controller.next}
        onTest={controller.connections.testRedis}
        testing={controller.connections.testingRedis}
      />
    );
  }
  if (controller.step === 'administrator') {
    return (
      <AdministratorStep
        advancedOpen={controller.advancedOpen}
        onBack={controller.back}
        onNext={controller.next}
        onToggleAdvanced={controller.toggleAdvanced}
      />
    );
  }
  if (controller.step === 'confirmation') {
    return (
      <InstallationConfirmation
        advancedOpen={controller.advancedOpen}
        advancedKeys={controller.advancedKeys}
        error={controller.installation.error}
        submitting={controller.installation.submitting}
        onBack={controller.back}
        onInstall={controller.installation.submit}
      />
    );
  }

  return <RestartWaitScreen controller={controller} />;
}

function RestartWaitScreen({
  controller,
}: Readonly<{ controller: ReturnType<typeof useSetupWizard> }>) {
  const { t } = useTranslate('setup');
  const unavailable = controller.restart.state === 'unavailable';

  return (
    <Stack alignItems="center" spacing={2} sx={{ py: 6, textAlign: 'center' }}>
      <Typography variant="h4">{t('status.restartWaitingTitle')}</Typography>
      <Typography color="text.secondary">{t('status.restartWaitingDescription')}</Typography>
      {unavailable ? <Alert severity="error">{t('status.restartUnavailable')}</Alert> : null}
      <Button variant="outlined" onClick={controller.restart.retry}>
        {t('actions.retry')}
      </Button>
    </Stack>
  );
}

function stepIndex(step: SetupWizardStep): number {
  const index = OPERATOR_STEPS.indexOf(step);
  return index === -1 ? OPERATOR_STEPS.length : index;
}

type SetupStateScreenProps = Readonly<{
  title: string;
  detail?: string;
  retryLabel?: string;
  onRetry?: () => void;
}>;

function SetupStateScreen({ title, detail, retryLabel, onRetry }: SetupStateScreenProps) {
  return (
    <Stack
      alignItems="center"
      justifyContent="center"
      spacing={2}
      sx={{ minHeight: '100vh', p: 3, textAlign: 'center' }}
    >
      <Typography variant="h5">{title}</Typography>
      {detail ? <Alert severity="error">{detail}</Alert> : null}
      {onRetry && retryLabel ? (
        <Button variant="contained" onClick={onRetry}>
          {retryLabel}
        </Button>
      ) : null}
    </Stack>
  );
}
