/* SHA-256's word schedule and rounds are defined with 32-bit bitwise operations. */
/* eslint-disable no-bitwise */

const SHA256_BLOCK_BYTES = 64;
const SHA256_LENGTH_OFFSET = 56;
const BITS_PER_BYTE = 8;
const UINT32_BASE = 0x100000000;

const INITIAL_STATE = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);

const ROUND_CONSTANTS = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);

export class Sha256 {
  private readonly state = new Uint32Array(INITIAL_STATE);
  private readonly buffer = new Uint8Array(SHA256_BLOCK_BYTES);
  private bufferLength = 0;
  private bytesHashed = 0;
  private finished = false;

  update(input: Uint8Array): this {
    if (this.finished) throw new Error('SHA-256 digest has already been finalized');
    this.bytesHashed += input.byteLength;
    let offset = 0;
    while (offset < input.byteLength) {
      const available = SHA256_BLOCK_BYTES - this.bufferLength;
      const length = Math.min(available, input.byteLength - offset);
      this.buffer.set(input.subarray(offset, offset + length), this.bufferLength);
      this.bufferLength += length;
      offset += length;
      if (this.bufferLength === SHA256_BLOCK_BYTES) this.processBlock(this.buffer);
    }
    return this;
  }

  digestHex(): string {
    this.finish();
    return Array.from(this.state, (word) => word.toString(16).padStart(8, '0')).join('');
  }

  private finish() {
    if (this.finished) return;
    const bitLength = this.bytesHashed * BITS_PER_BYTE;
    this.buffer[this.bufferLength] = 0x80;
    this.bufferLength += 1;
    if (this.bufferLength > SHA256_LENGTH_OFFSET) {
      this.buffer.fill(0, this.bufferLength);
      this.processBlock(this.buffer);
    }
    this.buffer.fill(0, this.bufferLength, SHA256_LENGTH_OFFSET);
    writeUint32(this.buffer, SHA256_LENGTH_OFFSET, Math.floor(bitLength / UINT32_BASE));
    writeUint32(this.buffer, SHA256_LENGTH_OFFSET + 4, bitLength >>> 0);
    this.processBlock(this.buffer);
    this.finished = true;
  }

  private processBlock(block: Uint8Array) {
    const words = createSchedule(block);
    let [a, b, c, d, e, f, g, h] = this.state;
    for (let index = 0; index < ROUND_CONSTANTS.length; index += 1) {
      const sum1 = (h + sigma1(e) + choose(e, f, g) + ROUND_CONSTANTS[index] + words[index]) >>> 0;
      const sum2 = (sigma0(a) + majority(a, b, c)) >>> 0;
      [a, b, c, d, e, f, g, h] = [sum1 + sum2, a, b, c, d + sum1, e, f, g].map(
        (value) => value >>> 0
      );
    }
    const result = [a, b, c, d, e, f, g, h];
    for (let index = 0; index < this.state.length; index += 1) {
      this.state[index] = (this.state[index] + result[index]) >>> 0;
    }
    this.bufferLength = 0;
  }
}

export async function sha256Blob(blob: Blob, signal?: AbortSignal): Promise<string> {
  signal?.throwIfAborted();
  const bytes = new Uint8Array(await blob.arrayBuffer());
  signal?.throwIfAborted();
  return new Sha256().update(bytes).digestHex();
}

export async function sha256File(
  file: File,
  options: Readonly<{
    chunkSize: number;
    signal?: AbortSignal;
    onProgress?: (processedBytes: number) => void;
  }>
): Promise<string> {
  if (!Number.isSafeInteger(options.chunkSize) || options.chunkSize <= 0) {
    throw new Error('SHA-256 chunk size must be a positive safe integer');
  }
  const hash = new Sha256();
  for (let offset = 0; offset < file.size; offset += options.chunkSize) {
    options.signal?.throwIfAborted();
    const end = Math.min(file.size, offset + options.chunkSize);
    const chunk = file.slice(offset, end);
    hash.update(new Uint8Array(await chunk.arrayBuffer()));
    options.onProgress?.(end);
  }
  options.signal?.throwIfAborted();
  return hash.digestHex();
}

export function sha256Text(value: string): string {
  return new Sha256().update(new TextEncoder().encode(value)).digestHex();
}

function createSchedule(block: Uint8Array): Uint32Array {
  const words = new Uint32Array(ROUND_CONSTANTS.length);
  for (let index = 0; index < 16; index += 1) {
    const offset = index * 4;
    words[index] =
      ((block[offset] << 24) |
        (block[offset + 1] << 16) |
        (block[offset + 2] << 8) |
        block[offset + 3]) >>>
      0;
  }
  for (let index = 16; index < words.length; index += 1) {
    words[index] =
      (words[index - 16] +
        smallSigma0(words[index - 15]) +
        words[index - 7] +
        smallSigma1(words[index - 2])) >>>
      0;
  }
  return words;
}

function writeUint32(target: Uint8Array, offset: number, value: number) {
  target[offset] = value >>> 24;
  target[offset + 1] = value >>> 16;
  target[offset + 2] = value >>> 8;
  target[offset + 3] = value;
}

const rotateRight = (value: number, bits: number) => (value >>> bits) | (value << (32 - bits));
const choose = (x: number, y: number, z: number) => (x & y) ^ (~x & z);
const majority = (x: number, y: number, z: number) => (x & y) ^ (x & z) ^ (y & z);
const sigma0 = (value: number) =>
  rotateRight(value, 2) ^ rotateRight(value, 13) ^ rotateRight(value, 22);
const sigma1 = (value: number) =>
  rotateRight(value, 6) ^ rotateRight(value, 11) ^ rotateRight(value, 25);
const smallSigma0 = (value: number) =>
  rotateRight(value, 7) ^ rotateRight(value, 18) ^ (value >>> 3);
const smallSigma1 = (value: number) =>
  rotateRight(value, 17) ^ rotateRight(value, 19) ^ (value >>> 10);
