'use client';

import type { UseFormReturn } from 'react-hook-form';
import type { SetupFormValues } from './form-values';

import { useState, useCallback } from 'react';

import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { installTaco } from 'src/entities/installation';

import { buildInstallationRequest } from './payload';
import { getDirtyAdvancedKeys } from './advanced-overrides';

type InstallationSubmitOptions = Readonly<{
  methods: UseFormReturn<SetupFormValues>;
  advancedOpen: boolean;
  onInstalled: () => void;
}>;

export function useInstallationSubmit({
  methods,
  advancedOpen,
  onInstalled,
}: InstallationSubmitOptions) {
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const submit = useCallback(async () => {
    const valid = await methods.trigger();
    if (!valid) return;

    setError(null);
    setSubmitting(true);
    try {
      await installTaco(
        buildInstallationRequest(methods.getValues(), {
          advancedKeys: getDirtyAdvancedKeys(methods.formState.dirtyFields.advanced, advancedOpen),
        })
      );
      onInstalled();
    } catch (submissionError) {
      setError(getErrorMessage(submissionError));
    } finally {
      setSubmitting(false);
    }
  }, [advancedOpen, methods, onInstalled]);

  return { error, submitting, submit };
}
