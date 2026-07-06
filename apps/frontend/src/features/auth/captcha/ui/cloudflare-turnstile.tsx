'use client';

import type { TurnstilePublicConfig } from '../model/types';

import Script from 'next/script';
import { useRef, useState, useEffect } from 'react';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';

const TURNSTILE_CONTAINER_ID_PREFIX = 'turnstile-captcha';

type TurnstileWidgetId = string;

type TurnstileApi = {
  render: (container: HTMLElement, options: TurnstileRenderOptions) => TurnstileWidgetId;
  reset: (widgetId: TurnstileWidgetId) => void;
  remove: (widgetId: TurnstileWidgetId) => void;
};

type TurnstileRenderOptions = {
  sitekey: string;
  theme: string;
  size: string;
  callback: (token: string) => void;
  'error-callback': () => void;
  'expired-callback': () => void;
  'timeout-callback': () => void;
};

type CloudflareTurnstileProps = {
  config: TurnstilePublicConfig;
  resetKey: number;
  onTokenChange: (token: string | null) => void;
};

declare global {
  interface Window {
    turnstile?: TurnstileApi;
  }
}

export function CloudflareTurnstile({ config, resetKey, onTokenChange }: CloudflareTurnstileProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const widgetIdRef = useRef<TurnstileWidgetId | null>(null);
  const containerId = useRef(`${TURNSTILE_CONTAINER_ID_PREFIX}-${crypto.randomUUID()}`);
  const [scriptReady, setScriptReady] = useState(false);

  useEffect(() => removeWidget(widgetIdRef), []);
  useEffect(() => resetWidget(widgetIdRef, onTokenChange), [onTokenChange, resetKey]);
  useEffect(
    () => renderWidget({ config, containerRef, widgetIdRef, scriptReady, onTokenChange }),
    [config, scriptReady, onTokenChange]
  );

  if (!config.site_key) {
    return <Alert severity="error">Cloudflare Turnstile site_key is required</Alert>;
  }

  return (
    <Box sx={{ width: 1 }}>
      <Script
        src={config.script_url}
        strategy="afterInteractive"
        onReady={() => setScriptReady(true)}
      />
      <Box id={containerId.current} ref={containerRef} sx={{ minHeight: 65 }} />
    </Box>
  );
}

type RenderWidgetOptions = {
  config: TurnstilePublicConfig;
  containerRef: React.RefObject<HTMLDivElement | null>;
  widgetIdRef: React.MutableRefObject<TurnstileWidgetId | null>;
  scriptReady: boolean;
  onTokenChange: (token: string | null) => void;
};

function renderWidget(options: RenderWidgetOptions) {
  const container = options.containerRef.current;
  const turnstile = window.turnstile;
  if (!options.scriptReady || !container || !turnstile || options.widgetIdRef.current) {
    return undefined;
  }

  options.onTokenChange(null);
  options.widgetIdRef.current = turnstile.render(
    container,
    renderOptions(options.config, options.onTokenChange)
  );

  return () => removeWidget(options.widgetIdRef)();
}

function renderOptions(
  config: TurnstilePublicConfig,
  onTokenChange: (token: string | null) => void
) {
  const resetToken = () => onTokenChange(null);

  return {
    sitekey: config.site_key,
    theme: config.theme,
    size: config.size,
    callback: (token: string) => onTokenChange(token || null),
    'error-callback': resetToken,
    'expired-callback': resetToken,
    'timeout-callback': resetToken,
  } satisfies TurnstileRenderOptions;
}

function resetWidget(
  widgetIdRef: React.MutableRefObject<TurnstileWidgetId | null>,
  onTokenChange: (token: string | null) => void
) {
  const widgetId = widgetIdRef.current;
  if (widgetId) {
    window.turnstile?.reset(widgetId);
  }
  onTokenChange(null);
}

function removeWidget(widgetIdRef: React.MutableRefObject<TurnstileWidgetId | null>) {
  return () => {
    const widgetId = widgetIdRef.current;
    if (widgetId) {
      window.turnstile?.remove(widgetId);
      widgetIdRef.current = null;
    }
  };
}
