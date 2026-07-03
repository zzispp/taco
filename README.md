# taco

A Rust + Next.js admin template for teams that want a clean backend foundation and a ready-to-use management console.

`taco` combines an Axum backend, a Next.js dashboard, PostgreSQL, Redis, SQLx migrations, and role-based access control in a single monorepo. The current template is focused on authentication and system administration: users, roles, API permissions, and menu management.

## Highlights

- Rust workspace backend with clear module boundaries
- Next.js 16 admin frontend with App Router and MUI 7
- JWT sign-in, sign-up, refresh, and current-user endpoints
- RBAC for users, roles, APIs, and menus
- OpenAPI and docs endpoints out of the box
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

## What Is Included

### Frontend routes

- `/`
- `/auth/sign-in`
- `/auth/sign-up`
- `/dashboard/admin/users`
- `/dashboard/admin/roles`
- `/dashboard/admin/apis`
- `/dashboard/admin/menus`

`/dashboard` redirects to `/dashboard/admin/users`.

### Backend public endpoints

- `GET /health`
- `GET /metrics`
- `GET /openapi.json`
- `GET /docs`

### Backend domain scope

- user management
- role management
- API permission management
- menu management
- authentication and authorization

## Repository Layout

```text
.
├── apps
│   ├── backend         # Axum app entry, composition root, migration commands
│   └── frontend        # Next.js admin app
├── crates
│   ├── config          # typed config loading and validation
│   ├── constants       # shared constants
│   ├── kernel          # small shared primitives
│   ├── rbac            # RBAC domain/application/infra/api
│   ├── storage         # database and storage foundation
│   ├── tracing         # tracing and metrics helpers
│   ├── types           # transport DTOs and shared request/response types
│   └── user            # user domain/application/infra/api
├── config              # YAML configuration
├── migrations          # SQLx migration files
├── compose.yaml        # local Postgres and Redis
└── justfile            # Rust-focused dev commands
```

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

