import type {
  AxiosInstance,
  AxiosResponse,
  RawAxiosHeaders,
  AxiosRequestConfig,
  RawAxiosRequestHeaders,
  InternalAxiosRequestConfig,
} from 'axios';

import axios, { AxiosHeaders } from 'axios';

import { isAuthSessionRejected, type NormalizedApiError } from './error-normalization';

const AUTHORIZATION_HEADER = 'Authorization';
const BEARER_SCHEME = 'Bearer';
const UNAUTHORIZED_STATUS = 401;

type RecoveryRequestConfig = InternalAxiosRequestConfig & {
  authSessionRecoveryRetried?: true;
  skipAuthSessionRecovery?: true;
};

type AuthSessionRecovery = Readonly<{
  refreshAccessToken: () => Promise<string | null>;
  onTerminalSessionRejected: () => Promise<void>;
}>;

let registeredRecovery: AuthSessionRecovery | undefined;
let refreshInFlight: Promise<string | null> | undefined;
let recoveredAccessToken: string | undefined;
let terminalSessionRejection: Promise<void> | undefined;

export function registerAuthSessionRecovery(recovery: AuthSessionRecovery): () => void {
  if (registeredRecovery) {
    throw new Error('Auth session recovery is already registered');
  }

  registeredRecovery = recovery;
  clearRecoveryState();

  return () => {
    if (registeredRecovery !== recovery) return;
    registeredRecovery = undefined;
    clearRecoveryState();
  };
}

export function markAuthSessionEstablished(): void {
  recoveredAccessToken = undefined;
  terminalSessionRejection = undefined;
}

function clearRecoveryState(): void {
  refreshInFlight = undefined;
  markAuthSessionEstablished();
}

export function skipAuthSessionRecovery(config: AxiosRequestConfig = {}): AxiosRequestConfig {
  return { ...config, skipAuthSessionRecovery: true } as AxiosRequestConfig;
}

export async function retryUnauthorizedRequest(
  client: AxiosInstance,
  error: unknown
): Promise<AxiosResponse | undefined> {
  const config = recoveryRequestConfig(error);
  if (!config || !shouldRecover(config, error)) return undefined;

  const token = await accessTokenForRetry(config, client);
  if (!token) return undefined;

  return client.request(createRetryConfig(config, token));
}

export async function notifyTerminalSessionRejection(
  error: unknown,
  normalizedError: NormalizedApiError
): Promise<void> {
  const config = recoveryRequestConfig(error);
  if (!config || config.skipAuthSessionRecovery || !requestHasAuthorization(config)) return;
  if (!isAuthSessionRejected(normalizedError) || !registeredRecovery) return;

  terminalSessionRejection ??= registeredRecovery.onTerminalSessionRejected();
  await terminalSessionRejection;
}

function recoveryRequestConfig(error: unknown): RecoveryRequestConfig | undefined {
  if (!axios.isAxiosError(error) || !error.config) return undefined;
  return error.config as RecoveryRequestConfig;
}

function shouldRecover(config: RecoveryRequestConfig, error: unknown): boolean {
  return Boolean(
    registeredRecovery &&
    errorStatus(error) === UNAUTHORIZED_STATUS &&
    requestHasAuthorization(config) &&
    !config.skipAuthSessionRecovery &&
    !config.authSessionRecoveryRetried
  );
}

async function accessTokenForRetry(
  config: RecoveryRequestConfig,
  client: AxiosInstance
): Promise<string | null> {
  const currentToken = currentAccessToken(config, client);
  if (currentToken) return currentToken;
  if (recoveredAccessToken && requestUsesDifferentToken(config, recoveredAccessToken)) {
    return recoveredAccessToken;
  }
  return refreshAccessToken();
}

function currentAccessToken(config: RecoveryRequestConfig, client: AxiosInstance): string | null {
  const currentAuthorization = authorizationValue(client.defaults.headers.common);
  const requestAuthorization = authorizationValue(config.headers);
  if (!currentAuthorization || currentAuthorization === requestAuthorization) return null;
  return accessTokenFromAuthorization(currentAuthorization);
}

function requestUsesDifferentToken(config: RecoveryRequestConfig, accessToken: string): boolean {
  return authorizationValue(config.headers) !== `${BEARER_SCHEME} ${accessToken}`;
}

async function refreshAccessToken(): Promise<string | null> {
  if (!refreshInFlight) {
    refreshInFlight = requestRefreshedAccessToken().finally(() => {
      refreshInFlight = undefined;
    });
  }
  return refreshInFlight;
}

async function requestRefreshedAccessToken(): Promise<string | null> {
  try {
    const accessToken = await registeredRecovery?.refreshAccessToken();
    recoveredAccessToken = accessToken || undefined;
    return recoveredAccessToken ?? null;
  } catch (error) {
    if (isAuthSessionRejected(error)) return null;
    throw error;
  }
}

function createRetryConfig(
  config: RecoveryRequestConfig,
  accessToken: string
): RecoveryRequestConfig {
  const headers = new AxiosHeaders(config.headers);
  headers.set(AUTHORIZATION_HEADER, `${BEARER_SCHEME} ${accessToken}`);

  return {
    ...config,
    headers,
    authSessionRecoveryRetried: true,
  };
}

function requestHasAuthorization(config: RecoveryRequestConfig): boolean {
  return Boolean(authorizationValue(config.headers));
}

function authorizationValue(
  headers: RawAxiosHeaders | RawAxiosRequestHeaders | AxiosHeaders | undefined
): string | undefined {
  const authorization = new AxiosHeaders(headers as RawAxiosHeaders | AxiosHeaders | undefined).get(
    AUTHORIZATION_HEADER
  );
  return typeof authorization === 'string' && authorization.trim() ? authorization : undefined;
}

function accessTokenFromAuthorization(authorization: string): string | null {
  const [scheme, token] = authorization.split(/\s+/, 2);
  return scheme === BEARER_SCHEME && token ? token : null;
}

function errorStatus(error: unknown): number | undefined {
  return axios.isAxiosError(error) ? error.response?.status : undefined;
}
