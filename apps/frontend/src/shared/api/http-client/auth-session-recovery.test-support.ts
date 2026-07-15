import type { AxiosResponse, AxiosHeaderValue, InternalAxiosRequestConfig } from 'axios';

import axios, { AxiosError, AxiosHeaders } from 'axios';

const UNAUTHORIZED = Object.freeze({
  code: 'unauthorized',
  message: '未登录或登录已失效',
  details: '未登录或登录已失效',
});

export function response<T>(config: InternalAxiosRequestConfig, data: T): AxiosResponse<T> {
  return {
    config,
    data,
    status: 200,
    statusText: '200',
    headers: {},
  };
}

export function unauthorized(config: InternalAxiosRequestConfig): AxiosError<typeof UNAUTHORIZED> {
  return new AxiosError(
    'Request failed with status code 401',
    AxiosError.ERR_BAD_REQUEST,
    config,
    undefined,
    {
      config,
      data: UNAUTHORIZED,
      status: 401,
      statusText: 'Unauthorized',
      headers: {},
    }
  );
}

export function authorization(config: InternalAxiosRequestConfig): string | undefined {
  const value = AxiosHeaders.from(config.headers).get('Authorization');
  return typeof value === 'string' ? value : undefined;
}

export function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

export function restoreAuthorization(original: AxiosHeaderValue | undefined): void {
  if (original === undefined) {
    delete axios.defaults.headers.common.Authorization;
    return;
  }
  axios.defaults.headers.common.Authorization = original;
}
