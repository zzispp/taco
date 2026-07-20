# Taco Frontend

The admin frontend is a Next.js application in the root pnpm workspace.

## Prerequisites

- Node.js 22+
- pnpm 10+

Install dependencies and run commands from the repository root:

```bash
pnpm install
pnpm dev:frontend
```

The local frontend listens on `http://localhost:8082`. Its development proxy
forwards same-origin `/api/*` requests to `http://localhost:3000` by default.
Use `localhost` for both applications so the strict refresh Cookie remains
same-site. A request through `127.0.0.1:8082` redirects to the canonical
localhost frontend origin.

## Process Environment

The frontend accepts environment variables only from its inherited process environment. `.env` and every `.env.*` file are unsupported; development and build fail explicitly when one is present in the repository root or `apps/frontend`.

Browser API requests always use same-origin `/api/*` paths. During standalone
development, Next.js proxies that path to `http://localhost:3000` by default;
set the server-only `TACO_DEV_BACKEND_URL` to use another backend origin.
Production uses `pnpm build:embedded` and does not run `next start`.

## Validation

```bash
pnpm --filter frontend test
pnpm lint:frontend
pnpm --filter frontend build:embedded
```
