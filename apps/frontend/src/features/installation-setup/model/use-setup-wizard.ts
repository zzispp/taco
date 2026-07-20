'use client';

import type { FieldPath, UseFormReturn } from 'react-hook-form';
import type { TranslateFn } from 'src/shared/i18n';
import type { SetupDefaults } from 'src/entities/installation';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useMemo, useState, useCallback } from 'react';

import { paths } from 'src/shared/routes/paths';
import { useRouter } from 'src/shared/routes/hooks';

import { createSetupFormSchema } from './schema';
import { useRestartWait } from './use-restart-wait';
import { useConnectionTests } from './use-connection-tests';
import { getDirtyAdvancedKeys } from './advanced-overrides';
import { useInstallationSubmit } from './use-installation-submit';
import { type SetupFormValues, createSetupFormValues } from './form-values';

export type SetupWizardStep = 'postgres' | 'redis' | 'administrator' | 'confirmation' | 'restart';

export function useSetupWizard(
  defaults: SetupDefaults,
  setupT: TranslateFn,
  messagesT: TranslateFn
) {
  const router = useRouter();
  const schema = useMemo(() => createSetupFormSchema(setupT, messagesT), [messagesT, setupT]);
  const methods = useForm<SetupFormValues>({
    defaultValues: createSetupFormValues(defaults),
    mode: 'onBlur',
    resolver: zodResolver(schema),
  });
  const [step, setStep] = useState<SetupWizardStep>('postgres');
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const redirectToSignIn = useCallback(() => router.replace(paths.auth.jwt.signIn), [router]);
  const restart = useRestartWait(redirectToSignIn);
  const beginRestart = useCallback(() => {
    setStep('restart');
    restart.start();
  }, [restart]);
  const connections = useConnectionTests({ methods, t: setupT });
  const installation = useInstallationSubmit({ methods, advancedOpen, onInstalled: beginRestart });
  const advancedKeys = getDirtyAdvancedKeys(methods.formState.dirtyFields.advanced, advancedOpen);
  const next = useNextStep({ step, setStep, connections, methods, advancedOpen });
  const back = useCallback(() => setStep(previousStep(step)), [step]);
  const toggleAdvanced = useCallback(
    () => toggleAdvancedSettings({ advancedOpen, methods, setAdvancedOpen }),
    [advancedOpen, methods]
  );

  return {
    methods,
    step,
    advancedOpen,
    advancedKeys,
    connections,
    installation,
    restart,
    next,
    back,
    toggleAdvanced,
  };
}

type NextStepOptions = Readonly<{
  step: SetupWizardStep;
  setStep: (step: SetupWizardStep) => void;
  connections: ReturnType<typeof useConnectionTests>;
  methods: UseFormReturn<SetupFormValues>;
  advancedOpen: boolean;
}>;

function useNextStep({ step, setStep, connections, methods, advancedOpen }: NextStepOptions) {
  return useCallback(async () => {
    if (step === 'postgres') {
      if (await connections.testPostgres()) setStep(nextStep(step));
      return;
    }
    if (step === 'redis') {
      if (await connections.testRedis()) setStep(nextStep(step));
      return;
    }
    if (step === 'administrator') {
      if (await methods.trigger(administratorValidationFields(advancedOpen))) {
        setStep(nextStep(step));
      }
      return;
    }
  }, [advancedOpen, connections, methods, setStep, step]);
}

const NEXT_STEP: Partial<Record<SetupWizardStep, SetupWizardStep>> = {
  postgres: 'redis',
  redis: 'administrator',
  administrator: 'confirmation',
};

const PREVIOUS_STEP: Partial<Record<SetupWizardStep, SetupWizardStep>> = {
  redis: 'postgres',
  administrator: 'redis',
  confirmation: 'administrator',
};

export function nextStep(step: SetupWizardStep): SetupWizardStep {
  const next = NEXT_STEP[step];
  if (next) return next;
  throw new Error(`Setup step "${step}" has no next step`);
}

export function previousStep(step: SetupWizardStep): SetupWizardStep {
  const previous = PREVIOUS_STEP[step];
  if (previous) return previous;
  throw new Error(`Setup step "${step}" has no previous step`);
}

export function administratorValidationFields(advancedOpen: boolean): FieldPath<SetupFormValues>[] {
  return advancedOpen ? ['administrator', 'advanced'] : ['administrator'];
}

type ToggleAdvancedSettingsOptions = Readonly<{
  advancedOpen: boolean;
  methods: ReturnType<typeof useForm<SetupFormValues>>;
  setAdvancedOpen: (value: boolean) => void;
}>;

function toggleAdvancedSettings({
  advancedOpen,
  methods,
  setAdvancedOpen,
}: ToggleAdvancedSettingsOptions) {
  if (advancedOpen) methods.resetField('advanced');
  setAdvancedOpen(!advancedOpen);
}
