FROM node:22-bookworm-slim AS frontend-build

WORKDIR /workspace

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY locale-contract.json ./locale-contract.json
COPY apps/frontend/package.json ./apps/frontend/package.json

RUN corepack enable && pnpm install --frozen-lockfile

COPY apps/frontend ./apps/frontend

RUN pnpm --filter frontend build:embedded \
    && node -e "const fs = require('node:fs'); const { locales } = require('./locale-contract.json'); for (const { code } of locales) { fs.accessSync('apps/frontend/out/' + code + '/index.html'); fs.accessSync('apps/frontend/out/' + code + '/error/404/index.html'); } fs.accessSync('apps/frontend/out/404.html');"

FROM rust:1.94-bookworm AS backend-build

WORKDIR /workspace

COPY . ./
COPY --from=frontend-build /workspace/apps/frontend/out ./apps/frontend/out

RUN cargo build --locked --release --package backend --bin taco --features embedded-frontend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm --recursive --force /var/lib/apt/lists/* \
    && mkdir --parents /app/config /app/local-data

COPY --from=backend-build /workspace/target/release/taco /usr/local/bin/taco

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/taco"]
