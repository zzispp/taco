# CAP Runtime Assets

These browser assets are pinned and served by the frontend so CAPTCHA solving does not depend on a runtime CDN request.

## Widget

- Package: `@cap.js/widget@0.1.56`
- License: `Apache-2.0`
- npm integrity: `sha512-j640dNNNIF8IWmwqmSx0ihgU8sz/6Jm9mHveeDWUk8aXVqFm+2TSsp5bawtMtgf0aa7rFkmT9p76jrqO1uSEpQ==`
- Upstream `cap.min.js` SHA-256: `296eb54ccfa39ba072fa68e81cb1013cb4d68cde2cfe22da0501543b2e7a2f9e`
- Vendored `cap.min.js` SHA-256: `e3b4b80c9ad27c48cd7fb67e363670b1fd930b950a3c59d6a6a01857c17e6409`

The vendored script has two deliberate changes: its default WASM and pako fallback URLs point to the same-origin assets listed below instead of jsDelivr. No solver logic is changed.

## WASM

- Package: `@cap.js/wasm@0.0.7`
- Package file: `browser/cap_wasm_bg.wasm`
- License: `Apache-2.0`
- npm integrity: `sha512-IgUjrPOUBaOjTp+BkrhfEBBeQ4An7fQiSWWezDy9Uvd+OdTYm4+h3AJU0j/CpHYayp7FltZU+UePC6p28oGQaw==`
- Vendored file SHA-256: `e4f3c00246a775193661f9277ca1288cd310a6514de166ecc2176ccd26fb06a9`

## Pako fallback

- Package: `pako@2.1.0`
- Package file: `dist/pako_inflate.min.js`
- License: `(MIT AND Zlib)`
- npm integrity: `sha512-w+eufiZ1WuJYgPXbV/PO3NCMEc3xqylkKHzp8bxp1uW4qaSNQUkwmLLEc3kKsfz8lpV1F8Ht3U1Cm+9Srog2ug==`
- Vendored file SHA-256: `fa226c8e1e3556993260e6a5c1fe94e225da59b3418a06811fdc51d308f8bb43`

## License files

- `licenses/Apache-2.0.txt` contains the Apache License 2.0 text used by both CAP packages.
- `licenses/cap-copyright.txt` preserves the upstream CAP copyright and license notice.
- `licenses/pako-2.1.0-MIT.txt` preserves pako's upstream MIT license.
- `licenses/pako-2.1.0-zlib.txt` preserves the zlib-derived code notice distributed by pako.
