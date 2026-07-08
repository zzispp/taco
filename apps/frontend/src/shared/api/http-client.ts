import type { AxiosRequestConfig, InternalAxiosRequestConfig } from 'axios';

import axios, { AxiosHeaders } from 'axios';

import { CONFIG } from 'src/shared/config';
import { storageConfig } from 'src/shared/i18n/locales-config';

// ----------------------------------------------------------------------

type ApiErrorPayload = {
  code?: unknown;
  message?: unknown;
  details?: unknown;
};

export type NormalizedApiError = Error & {
  status?: number;
  code?: string;
  details?: string;
};

type AuthSessionRejectedListener = (error: NormalizedApiError) => void;

const authSessionRejectedListeners = new Set<AuthSessionRejectedListener>();

const axiosInstance = axios.create({
  baseURL: CONFIG.serverUrl,
  headers: {
    'Content-Type': 'application/json',
  },
});

axiosInstance.interceptors.request.use((config) => applyAcceptLanguageHeader(config));

axiosInstance.interceptors.response.use(
  (response) => response,
  (error) => {
    const normalizedError = normalizeApiError(error);
    if (shouldNotifyAuthSessionRejected(error, normalizedError)) {
      notifyAuthSessionRejected(normalizedError);
    }
    return Promise.reject(normalizedError);
  }
);

export default axiosInstance;

// ----------------------------------------------------------------------

export function mapLanguageToAcceptLanguage(lang?: string | null): string | undefined {
  if (!lang) {
    return undefined;
  }

  const lower = lang.trim().toLowerCase().replace('_', '-');

  if (
    lower === 'tw' ||
    lower.startsWith('zh-tw') ||
    lower.startsWith('zh-hk') ||
    lower.startsWith('zh-hant')
  ) {
    return 'zh-TW';
  }

  if (
    lower === 'cn' ||
    lower === 'zh' ||
    lower.startsWith('zh-cn') ||
    lower.startsWith('zh-hans')
  ) {
    return 'zh-CN';
  }

  if (lower === 'en' || lower.startsWith('en-')) {
    return 'en';
  }

  return undefined;
}

export function normalizeApiError(error: unknown): NormalizedApiError {
  const response = axios.isAxiosError(error) ? error.response : undefined;
  const data = response?.data as ApiErrorPayload | undefined;
  const details = stringValue(data?.details);
  const message = details ?? stringValue(data?.message) ?? axiosErrorMessage(error);
  const normalizedError = new Error(message) as NormalizedApiError;

  Object.assign(normalizedError, {
    status: response?.status,
    code: stringValue(data?.code),
    details,
  });

  return normalizedError;
}

export function subscribeAuthSessionRejected(listener: AuthSessionRejectedListener) {
  authSessionRejectedListeners.add(listener);
  return () => {
    authSessionRejectedListeners.delete(listener);
  };
}

export function isAuthSessionRejected(error: unknown): boolean {
  if (typeof error !== 'object' || error === null) {
    return false;
  }

  const { status, code } = error as { status?: number; code?: string };
  return status === 401 || code === 'unauthorized';
}

export const fetcher = async <T = unknown>(
  args: string | [string, AxiosRequestConfig]
): Promise<T> => {
  try {
    const [url, config] = Array.isArray(args) ? args : [args, {}];

    const res = await axiosInstance.get<T>(url, config);

    return res.data;
  } catch (error) {
    if (!isAuthSessionRejected(error)) {
      console.error('Fetcher failed:', error);
    }
    throw error;
  }
};

function applyAcceptLanguageHeader(config: InternalAxiosRequestConfig): InternalAxiosRequestConfig {
  const acceptLanguage = storedAcceptLanguage();

  if (!acceptLanguage) {
    return config;
  }

  const headers = AxiosHeaders.from(config.headers);
  headers.set('Accept-Language', acceptLanguage);
  config.headers = headers;

  return config;
}

function storedAcceptLanguage(): string | undefined {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return mapLanguageToAcceptLanguage(localStorage.getItem(storageConfig.localStorage.key));
}

function shouldNotifyAuthSessionRejected(error: unknown, normalizedError: NormalizedApiError) {
  return isAuthSessionRejected(normalizedError) && requestHasAuthorizationHeader(error);
}

function requestHasAuthorizationHeader(error: unknown): boolean {
  if (!axios.isAxiosError(error)) {
    return false;
  }

  const authorization = AxiosHeaders.from(error.config?.headers).get('Authorization');
  return typeof authorization === 'string' ? authorization.trim().length > 0 : Boolean(authorization);
}

function notifyAuthSessionRejected(error: NormalizedApiError) {
  authSessionRejectedListeners.forEach((listener) => listener(error));
}

function axiosErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }

  return 'Something went wrong!';
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}
