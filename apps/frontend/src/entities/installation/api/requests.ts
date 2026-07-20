import type {
  SetupDefaults,
  InstallationState,
  InstallationRequest,
  RedisConnectionInput,
  PostgresConnectionInput,
} from '../model/types';

import axios from 'src/shared/api/http-client';

import { installationEndpoints } from './endpoints';
import {
  parseSetupDefaults,
  parseConnectionTest,
  parseInstallationState,
  parseInstallationComplete,
} from '../model/types';

export async function probeInstallationStatus(): Promise<InstallationState> {
  const response = await axios.get<unknown>(installationEndpoints.status);
  return parseInstallationState(response.data);
}

export async function getSetupDefaults(): Promise<SetupDefaults> {
  const response = await axios.get<unknown>(installationEndpoints.defaults);
  return parseSetupDefaults(response.data);
}

export async function testPostgresConnection(input: PostgresConnectionInput): Promise<void> {
  const response = await axios.post<unknown>(installationEndpoints.postgresTest, input);
  parseConnectionTest(response.data);
}

export async function testRedisConnection(input: RedisConnectionInput): Promise<void> {
  const response = await axios.post<unknown>(installationEndpoints.redisTest, input);
  parseConnectionTest(response.data);
}

export async function installTaco(input: InstallationRequest): Promise<void> {
  const response = await axios.post<unknown>(installationEndpoints.install, input);
  parseInstallationComplete(response.data);
}
