import type { NextConfig } from 'next';

import { DEFAULT_SERVER_URL } from './src/shared/config/server-url';

// ----------------------------------------------------------------------

/**
 * Static Exports in Next.js
 *
 * 1. Set `isStaticExport = true` in `next.config.{mjs|ts}`.
 * 2. This allows `generateStaticParams()` to pre-render dynamic routes at build time.
 *
 * For more details, see:
 * https://nextjs.org/docs/app/building-your-application/deploying/static-exports
 *
 * NOTE: Remove all "generateStaticParams()" functions if not using static exports.
 */
const isStaticExport = false;
const LOCAL_FRONTEND_ORIGIN = 'http://localhost:8082';
const LOOPBACK_HOST_PATTERN = '127\\.0\\.0\\.1';
const TURNSTILE_ORIGIN = 'https://challenges.cloudflare.com';
const WASM_EVAL_SOURCE = "'wasm-unsafe-eval'";

const contentSecurityPolicy = [
  "default-src 'self'",
  "base-uri 'self'",
  `connect-src 'self' ${backendOrigin()} ${TURNSTILE_ORIGIN}`,
  "font-src 'self' data:",
  "form-action 'self'",
  "frame-ancestors 'none'",
  `frame-src ${TURNSTILE_ORIGIN}`,
  "img-src 'self' blob: data: https:",
  "manifest-src 'self'",
  "media-src 'self' blob:",
  "object-src 'none'",
  `script-src 'self' 'unsafe-inline' ${WASM_EVAL_SOURCE}${process.env.NODE_ENV === 'development' ? " 'unsafe-eval'" : ''} ${TURNSTILE_ORIGIN}`,
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
  output: isStaticExport ? 'export' : undefined,
  env: {
    BUILD_STATIC_EXPORT: JSON.stringify(isStaticExport),
  },
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

function backendOrigin() {
  return new URL(process.env.NEXT_PUBLIC_SERVER_URL ?? DEFAULT_SERVER_URL).origin;
}

export default nextConfig;
