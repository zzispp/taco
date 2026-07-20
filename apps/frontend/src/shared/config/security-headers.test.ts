import { it, expect, describe } from 'vitest';

import { CONFIG } from './index';
import nextConfig, { NEXT_SECURITY_HEADERS } from '../../../next.config';

describe('Next security headers', () => {
  it('sets browser document protections for every frontend route', () => {
    const headers = Object.fromEntries(NEXT_SECURITY_HEADERS.map(({ key, value }) => [key, value]));

    expect(headers['Content-Security-Policy']).toContain("frame-ancestors 'none'");
    expect(headers['X-Frame-Options']).toBe('DENY');
    expect(headers['Referrer-Policy']).toBe('no-referrer');
    expect(headers['Permissions-Policy']).toBe('camera=(), microphone=(), geolocation=()');
    expect(headers['X-Content-Type-Options']).toBe('nosniff');
  });

  it('does not claim proxy-owned HSTS responsibility', () => {
    expect(NEXT_SECURITY_HEADERS.map(({ key }) => key)).not.toContain('Strict-Transport-Security');
  });

  it('permits same-origin CAP WASM compilation without allowing its CDN', () => {
    const contentSecurityPolicy = NEXT_SECURITY_HEADERS.find(
      ({ key }) => key === 'Content-Security-Policy'
    )?.value;

    expect(contentSecurityPolicy).toContain("script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'");
    expect(contentSecurityPolicy).not.toContain('cdn.jsdelivr.net');
  });

  it('uses a canonical localhost frontend origin and same-origin API policy in development', async () => {
    expect(CONFIG).not.toHaveProperty('serverUrl');
    expect(CONFIG.assetsDir).toBe('');
    expect(nextConfig.allowedDevOrigins).toBeUndefined();
    expect(await nextConfig.redirects?.()).toEqual([
      {
        source: '/:path*',
        has: [{ type: 'host', value: '127\\.0\\.0\\.1' }],
        destination: 'http://localhost:8082/:path*',
        permanent: false,
      },
    ]);
  });

  it('proxies browser API requests and uploaded assets to the development backend', async () => {
    expect(nextConfig.skipTrailingSlashRedirect).toBe(true);
    await expect(nextConfig.rewrites?.()).resolves.toEqual([
      {
        source: '/api/:path*',
        destination: 'http://localhost:3000/api/:path*',
      },
      {
        source: '/uploads/:path*',
        destination: 'http://localhost:3000/uploads/:path*',
      },
    ]);
  });

  it('does not treat arbitrary characters as dots in the loopback host matcher', async () => {
    const [redirect] = (await nextConfig.redirects?.()) ?? [];
    const hostPattern = redirect?.has?.[0]?.value;

    expect(hostPattern).toBe('127\\.0\\.0\\.1');
    expect(new RegExp(`^${hostPattern}$`).test('127.0.0.1')).toBe(true);
    expect(new RegExp(`^${hostPattern}$`).test('127x0x0x1')).toBe(false);
  });

  it('applies the header set to every frontend route', async () => {
    await expect(nextConfig.headers?.()).resolves.toEqual([
      { source: '/:path*', headers: NEXT_SECURITY_HEADERS },
    ]);
  });

  it('does not permit removed Turnstile browser origins', () => {
    const contentSecurityPolicy = NEXT_SECURITY_HEADERS.find(
      ({ key }) => key === 'Content-Security-Policy'
    )?.value;

    expect(contentSecurityPolicy).not.toContain('challenges.cloudflare.com');
    expect(contentSecurityPolicy).toContain("connect-src 'self'");
  });
});
