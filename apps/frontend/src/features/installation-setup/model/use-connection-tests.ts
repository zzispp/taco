'use client';

import type { UseFormReturn } from 'react-hook-form';
import type { TranslateFn } from 'src/shared/i18n';
import type { SetupFormValues } from './form-values';

import { useState, useCallback } from 'react';

import { toast } from 'src/shared/ui/snackbar';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { testRedisConnection, testPostgresConnection } from 'src/entities/installation';

import { buildInstallationRequest } from './payload';

const POSTGRES_FIELDS = [
  'postgres.host',
  'postgres.port',
  'postgres.username',
  'postgres.password',
  'postgres.database',
] as const;

const REDIS_FIELDS = ['redis.host', 'redis.port', 'redis.database'] as const;

type ConnectionTestOptions = Readonly<{
  methods: UseFormReturn<SetupFormValues>;
  t: TranslateFn;
}>;

export function useConnectionTests({ methods, t }: ConnectionTestOptions) {
  const [testingPostgres, setTestingPostgres] = useState(false);
  const [testingRedis, setTestingRedis] = useState(false);
  const testPostgres = useCallback(
    () => runPostgresTest(methods, t, setTestingPostgres),
    [methods, t]
  );
  const testRedis = useCallback(() => runRedisTest(methods, t, setTestingRedis), [methods, t]);

  return { testingPostgres, testingRedis, testPostgres, testRedis };
}

async function runPostgresTest(
  methods: UseFormReturn<SetupFormValues>,
  t: TranslateFn,
  setTesting: (value: boolean) => void
): Promise<boolean> {
  const valid = await methods.trigger(POSTGRES_FIELDS);
  if (!valid) return false;

  setTesting(true);
  try {
    await testPostgresConnection(requestFromValues(methods.getValues()).postgres);
    toast.success(t('status.connectionSucceeded'));
    return true;
  } catch (error) {
    toast.error(getErrorMessage(error));
    return false;
  } finally {
    setTesting(false);
  }
}

async function runRedisTest(
  methods: UseFormReturn<SetupFormValues>,
  t: TranslateFn,
  setTesting: (value: boolean) => void
): Promise<boolean> {
  const valid = await methods.trigger(REDIS_FIELDS);
  if (!valid) return false;

  setTesting(true);
  try {
    await testRedisConnection(requestFromValues(methods.getValues()).redis);
    toast.success(t('status.connectionSucceeded'));
    return true;
  } catch (error) {
    toast.error(getErrorMessage(error));
    return false;
  } finally {
    setTesting(false);
  }
}

function requestFromValues(values: SetupFormValues) {
  return buildInstallationRequest(values, { advancedKeys: [] });
}
