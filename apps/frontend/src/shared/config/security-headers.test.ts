import { it, expect, describe } from 'vitest';

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

  it('applies the header set to every frontend route', async () => {
    await expect(nextConfig.headers?.()).resolves.toEqual([
      { source: '/:path*', headers: NEXT_SECURITY_HEADERS },
    ]);
  });
});
