# taco

`taco` is a Rust + Next.js admin monorepo with a DDD + Clean Architecture backend and a Feature-Sliced Design frontend.

The backend is centered on bounded contexts for users and RBAC. The frontend is already refactored into FSD layers under `apps/frontend/src` and keeps the existing routes and API contracts stable.

## Highlights

- Rust backend organized by bounded context and Clean Architecture layers
- Next.js 16 frontend organized by FSD layers instead of template-style technical folders
- JWT sign-in, sign-up, refresh, and current-user endpoints
- RBAC for users, roles, API permissions, and menus
- PostgreSQL persistence with SQLx migrations
- Redis-backed RBAC cache rebuild on startup
- pnpm workspace + Cargo workspace in one repository

## Stack

### Backend

- Rust
- Axum
- SQLx
- PostgreSQL
- Redis
- Utoipa / Scalar

### Frontend

- Next.js 16
- React 19
- TypeScript
- MUI 7
- Axios
- SWR

## Architecture

### Backend: DDD + Clean Architecture

The backend keeps business logic inside domain crates and uses `apps/backend` as the composition root.

- `apps/backend`: app entry, dependency wiring, HTTP bootstrap, migration commands
- `crates/user`: user bounded context with `domain`, `application`, `infra`, `api`
- `crates/rbac`: RBAC bounded context with `domain`, `application`, `infra`, `api`
- `crates/config`: typed config loading and validation
- `crates/storage`: database and storage foundation
- `crates/types`: shared transport DTOs and request/response types
- `crates/constants`, `crates/kernel`, `crates/tracing`: shared support crates

Backend rule: user and RBAC business rules must stay in their owning domain crates. Do not push domain logic down into `apps/backend` or generic crates.

### Frontend: FSD on top of Next.js App Router

The frontend follows Feature-Sliced Design inside `apps/frontend/src`.

```text
apps/frontend/src
├── app          # Next.js App Router entry + FSD app layer
├── pages-layer  # page composition layer; named this way because Next.js reserves src/pages
├── widgets      # page blocks and layout/business containers
├── features     # user actions and use-case level UI behavior
├── entities     # business entities, entity UI, entity queries/types
└── shared       # pure shared infrastructure, UI, config, routes, theme, assets
```

Layer dependency direction is fixed:

- `app -> pages-layer/widgets/features/entities/shared`
- `pages-layer -> widgets/features/entities/shared`
- `widgets -> features/entities/shared`
- `features -> entities/shared`
- `entities -> shared`
- `shared -> shared`

Important frontend rule: `src/app/**/page.tsx` stays thin and delegates page composition to `src/pages-layer/**`. Business logic must not drift back into template-era top-level directories.

## Current Frontend Routes

- `/`
- `/auth/sign-in`
- `/auth/sign-up`
- `/dashboard/admin/users`
- `/dashboard/admin/roles`
- `/dashboard/admin/apis`
- `/dashboard/admin/menus`
- `/error/403`
- `/error/404`
- `/error/500`

`/dashboard` redirects to `/dashboard/admin/users`.

## Backend Public Endpoints

- `GET /health`
- `GET /metrics`
- `GET /openapi.json`
- `GET /docs`

## Backend Domain Scope

- authentication and authorization
- user management
- role management
- API permission management
- menu management

## Repository Layout

```text
.
├── apps
│   ├── backend         # Axum app entry and composition root
│   └── frontend        # Next.js admin app
├── crates
│   ├── config          # typed config loading and validation
│   ├── constants       # shared constants
│   ├── kernel          # shared low-level primitives
│   ├── rbac            # RBAC bounded context
│   ├── storage         # database and storage foundation
│   ├── tracing         # tracing and metrics helpers
│   ├── types           # shared transport DTOs
│   └── user            # user bounded context
├── config              # YAML runtime configuration
├── migrations          # SQLx migration files
├── compose.yaml        # local Postgres and Redis
└── justfile            # Rust-focused dev commands
```

## Architecture Guardrails

These rules are part of the repository contract:

- Frontend must stay inside the current FSD layers under `apps/frontend/src`.
- Do not reintroduce top-level template directories such as `sections`, `layouts`, `actions`, `types`, `components`, `routes`, `locales`, `theme`, `assets`, `lib`, `auth`, or `global-config` as parallel business structure.
- Backend must stay inside DDD + Clean Architecture boundaries.
- `apps/backend` is not a domain bucket; it is the composition root.
- Generic crates such as `config`, `storage`, `types`, `constants`, `kernel`, and `tracing` are not substitutes for bounded contexts.

## Quick Start

### Prerequisites

- Rust toolchain
- Node.js 22+
- pnpm 10+
- Docker

### 1. Install dependencies

```bash
pnpm install
```

### 2. Start infrastructure

```bash
docker compose up -d
```

This starts:

- PostgreSQL on `localhost:5433`
- Redis on `localhost:6380`

### 3. Apply database migrations

```bash
cargo run -p backend -- migration up
```

### 4. Start the backend

```bash
cargo run -p backend
```

Backend default address:

- `http://127.0.0.1:3000`

### 5. Start the frontend

```bash
pnpm dev:frontend
```

Frontend default address:

- `http://127.0.0.1:8082`

By default, the frontend calls the backend at `http://127.0.0.1:3000`. Override it with `NEXT_PUBLIC_SERVER_URL` when needed.

## Useful Commands

### Rust workspace

```bash
just check
just build
just test
just backend-migration "status"
just backend-migration "fresh"
```

### Frontend workspace

```bash
pnpm dev:frontend
pnpm build:frontend
pnpm lint:frontend
```

## Configuration

Runtime configuration lives in `config/config.yaml`.

Important areas:

- server host and port
- PostgreSQL connection
- Redis connection
- JWT secret and token TTL
- auth whitelist
- CORS
- HTTP timeout and compression
- tracing and file logging

Before deploying anywhere beyond local development, review and change:

- database credentials
- Redis connection settings
- JWT secret
- admin seed settings
- CORS policy

## Notes

- Migrations are explicit. Startup does not create or update schema automatically.
- RBAC cache is rebuilt during backend startup.
- The baseline data seeds roles, API permissions, menu sections, and menu items for the admin console.
