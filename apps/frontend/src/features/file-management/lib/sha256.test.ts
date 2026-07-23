import { it, expect, describe } from 'vitest';

import { Sha256, sha256Blob, sha256Text } from './sha256';

describe('file upload SHA-256', () => {
  it('matches the standard empty and abc vectors', () => {
    expect(sha256Text('')).toBe('e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
    expect(sha256Text('abc')).toBe(
      'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'
    );
  });

  it('keeps the same digest across incremental chunks', () => {
    const hash = new Sha256();
    hash.update(new TextEncoder().encode('a'));
    hash.update(new TextEncoder().encode('b'));
    hash.update(new TextEncoder().encode('c'));
    expect(hash.digestHex()).toBe(sha256Text('abc'));
  });

  it('matches the standard multi-block padding vector', () => {
    const value = 'abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq';
    expect(sha256Text(value)).toBe(
      '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1'
    );
  });

  it('hashes a blob without converting the whole managed file', async () => {
    const digest = await sha256Blob(new Blob(['hello world']));
    expect(digest).toBe('b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9');
  });
});
