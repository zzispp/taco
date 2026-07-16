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

The local frontend listens on `http://localhost:8082` and calls `http://localhost:3000` by default. Use `localhost` for both applications so the strict refresh Cookie remains same-site. A request through `127.0.0.1:8082` redirects to the canonical localhost frontend Origin.

## Process Environment

The frontend accepts environment variables only from its inherited process environment. `.env` and every `.env.*` file are unsupported; development, build, and start fail explicitly when one is present in the repository root or `apps/frontend`.

`NEXT_PUBLIC_SERVER_URL` optionally selects a different backend Origin. The frontend and backend must remain in the same schemeful site, and non-local deployments must use HTTPS.

## Validation

```bash
pnpm --filter frontend test
pnpm lint:frontend
pnpm build:frontend
```
