import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import type { NextConfig } from 'next';

import { assertNoEnvironmentFiles } from './src/shared/config/environment-files';

// ----------------------------------------------------------------------

const FRONTEND_ROOT = fileURLToPath(new URL('.', import.meta.url));
const WORKSPACE_ROOT = resolve(FRONTEND_ROOT, '../..');

assertNoEnvironmentFiles([WORKSPACE_ROOT, FRONTEND_ROOT]);

// ----------------------------------------------------------------------

const STATIC_EXPORT_ENV_VALUE = 'true';
const DEFAULT_DEVELOPMENT_BACKEND_URL = 'http://localhost:3000';
const isStaticExport = process.env.TACO_STATIC_EXPORT === STATIC_EXPORT_ENV_VALUE;
const LOCAL_FRONTEND_ORIGIN = 'http://localhost:8082';
const LOOPBACK_HOST_PATTERN = '127\\.0\\.0\\.1';
const WASM_EVAL_SOURCE = "'wasm-unsafe-eval'";

const contentSecurityPolicy = [
  "default-src 'self'",
  "base-uri 'self'",
  "connect-src 'self'",
  "font-src 'self' data:",
  "form-action 'self'",
  "frame-ancestors 'none'",
  "img-src 'self' blob: data: https:",
  "manifest-src 'self'",
  "media-src 'self' blob:",
  "object-src 'none'",
  `script-src 'self' 'unsafe-inline' ${WASM_EVAL_SOURCE}${process.env.NODE_ENV === 'development' ? " 'unsafe-eval'" : ''}`,
  "style-src 'self' 'unsafe-inline'",
  "worker-src 'self' blob:",
].join('; ');

export const NEXT_SECURITY_HEADERS = [
  { key: 'Content-Security-Policy', value: contentSecurityPolicy },
  { key: 'X-Frame-Options', value: 'DENY' },
  { key: 'Referrer-Policy', value: 'no-referrer' },
  { key: 'Permissions-Policy', value: 'camera=(), microphone=(), geolocation=()' },
  { key: 'X-Content-Type-Options', value: 'nosniff' },
];

// ----------------------------------------------------------------------

const nextConfig: NextConfig = {
  trailingSlash: true,
  skipTrailingSlashRedirect: true,
  output: isStaticExport ? 'export' : undefined,
  ...developmentServerConfig(),
  // Without --turbopack (next dev)
  webpack(config) {
    config.module.rules.push({
      test: /\.svg$/,
      use: ['@svgr/webpack'],
    });

    return config;
  },
  // With --turbopack (next dev --turbopack)
  turbopack: {
    rules: {
      '*.svg': {
        loaders: ['@svgr/webpack'],
        as: '*.js',
      },
    },
  },
};

function developmentServerConfig(): Partial<NextConfig> {
  if (isStaticExport) {
    return {};
  }

  return {
    async headers() {
      return [{ source: '/:path*', headers: NEXT_SECURITY_HEADERS }];
    },
    async redirects() {
      return [
        {
          source: '/:path*',
          has: [{ type: 'host', value: LOOPBACK_HOST_PATTERN }],
          destination: `${LOCAL_FRONTEND_ORIGIN}/:path*`,
          permanent: false,
        },
      ];
    },
    async rewrites() {
      return [
        {
          source: '/api/:path*',
          destination: `${developmentBackendUrl()}/api/:path*`,
        },
      ];
    },
  };
}

function developmentBackendUrl(): string {
  const configuredUrl = process.env.TACO_DEV_BACKEND_URL ?? DEFAULT_DEVELOPMENT_BACKEND_URL;
  const parsedUrl = new URL(configuredUrl);

  if (!['http:', 'https:'].includes(parsedUrl.protocol)) {
    throw new Error('TACO_DEV_BACKEND_URL must use http or https');
  }
  if (parsedUrl.pathname !== '/' || parsedUrl.search || parsedUrl.hash) {
    throw new Error('TACO_DEV_BACKEND_URL must be an origin without a path, query, or fragment');
  }

  return parsedUrl.origin;
}

export default nextConfig;
