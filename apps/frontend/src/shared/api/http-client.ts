import type { AxiosRequestConfig, InternalAxiosRequestConfig } from 'axios';

import axios, { AxiosHeaders } from 'axios';

import { storageConfig } from 'src/shared/i18n/locales-config';
import { toBackendAcceptLanguage } from 'src/shared/i18n/language';

import { isAuthSessionRejected, normalizeApiErrorAsync } from './http-client/error-normalization';
import {
  retryUnauthorizedRequest,
  notifyTerminalSessionRejection,
} from './http-client/auth-session-recovery';

export { toBackendAcceptLanguage as mapLanguageToAcceptLanguage } from 'src/shared/i18n/language';
export {
  skipAuthSessionRecovery,
  markAuthSessionEstablished,
  registerAuthSessionRecovery,
} from './http-client/auth-session-recovery';
export {
  normalizeApiError,
  isNormalizedApiError,
  isAuthSessionRejected,
  normalizeApiErrorAsync,
  type NormalizedApiError,
} from './http-client/error-normalization';

// ----------------------------------------------------------------------

const axiosInstance = axios.create({
  withCredentials: true,
  headers: {
    'Content-Type': 'application/json',
  },
});

axiosInstance.interceptors.request.use((config) => applyAcceptLanguageHeader(config));

axiosInstance.interceptors.response.use(
  (response) => response,
  async (error) => {
    const recoveredResponse = await retryUnauthorizedRequest(axiosInstance, error);
    if (recoveredResponse) return recoveredResponse;

    const normalizedError = await normalizeApiErrorAsync(error);
    await notifyTerminalSessionRejection(error, normalizedError);
    return Promise.reject(normalizedError);
  }
);

export default axiosInstance;

// ----------------------------------------------------------------------

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

  return toBackendAcceptLanguage(localStorage.getItem(storageConfig.localStorage.key));
}
