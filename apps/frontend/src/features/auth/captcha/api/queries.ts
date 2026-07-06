'use client';

import type { CaptchaConfig } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';

import { captchaEndpoints } from './endpoints';

export function useCaptchaConfig() {
  return useSWR<CaptchaConfig>(captchaEndpoints.config, fetcher, {
    revalidateOnFocus: false,
  });
}
