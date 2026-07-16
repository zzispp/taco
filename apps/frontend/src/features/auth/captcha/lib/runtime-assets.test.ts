import { join } from 'node:path';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { it, expect, describe } from 'vitest';

import { CAP_RUNTIME_ASSETS } from './runtime-assets';

const PUBLIC_ROOT = fileURLToPath(new URL('../../../../../public/', import.meta.url));
const EXPECTED_WIDGET_BYTE_LENGTH = 43_688;
const EXPECTED_WIDGET_SHA256 = 'e3b4b80c9ad27c48cd7fb67e363670b1fd930b950a3c59d6a6a01857c17e6409';
const EXPECTED_WASM_BYTE_LENGTH = 22_608;
const EXPECTED_WASM_SHA256 = 'e4f3c00246a775193661f9277ca1288cd310a6514de166ecc2176ccd26fb06a9';
const EXPECTED_PAKO_BYTE_LENGTH = 21_479;
const EXPECTED_PAKO_SHA256 = 'fa226c8e1e3556993260e6a5c1fe94e225da59b3418a06811fdc51d308f8bb43';
const UPSTREAM_WASM_URL =
  'https://cdn.jsdelivr.net/npm/@cap.js/wasm@0.0.7/browser/cap_wasm_bg.wasm';
const UPSTREAM_PAKO_URL =
  'https://cdn.jsdelivr.net/npm/pako@2.1.0/dist/pako_inflate.min.js';

describe('CAP runtime assets', () => {
  it('pins the reviewed same-origin widget build', () => {
    const widget = readFileSync(publicAssetPath(CAP_RUNTIME_ASSETS.widgetScriptUrl));

    expect(widget.byteLength).toBe(EXPECTED_WIDGET_BYTE_LENGTH);
    expect(createHash('sha256').update(widget).digest('hex')).toBe(EXPECTED_WIDGET_SHA256);
  });

  it('vendors the exact browser WASM from @cap.js/wasm 0.0.7', () => {
    const wasm = readFileSync(publicAssetPath(CAP_RUNTIME_ASSETS.wasmUrl));

    expect(wasm.byteLength).toBe(EXPECTED_WASM_BYTE_LENGTH);
    expect(createHash('sha256').update(wasm).digest('hex')).toBe(EXPECTED_WASM_SHA256);
  });

  it('binds the vendored widget to the paired same-origin WASM', () => {
    const widget = readFileSync(publicAssetPath(CAP_RUNTIME_ASSETS.widgetScriptUrl), 'utf8');
    const localWasmDefault = `window.CAP_CUSTOM_WASM_URL||"${CAP_RUNTIME_ASSETS.wasmUrl}"`;

    expect(widget).toContain(localWasmDefault);
    expect(widget).not.toContain(UPSTREAM_WASM_URL);
  });

  it('vendors the exact pako fallback used when DecompressionStream is unavailable', () => {
    const pako = readFileSync(publicAssetPath(CAP_RUNTIME_ASSETS.pakoScriptUrl));

    expect(pako.byteLength).toBe(EXPECTED_PAKO_BYTE_LENGTH);
    expect(createHash('sha256').update(pako).digest('hex')).toBe(EXPECTED_PAKO_SHA256);
  });

  it('binds the pako fallback to the same frontend origin', () => {
    const widget = readFileSync(publicAssetPath(CAP_RUNTIME_ASSETS.widgetScriptUrl), 'utf8');
    const localPakoDefault = `window.CAP_PAKO_URL||"${CAP_RUNTIME_ASSETS.pakoScriptUrl}"`;

    expect(widget).toContain(localPakoDefault);
    expect(widget).not.toContain(UPSTREAM_PAKO_URL);
  });
});

function publicAssetPath(url: string) {
  return join(PUBLIC_ROOT, url.slice(1));
}
