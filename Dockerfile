FROM node:22-bookworm-slim AS frontend-build

WORKDIR /workspace

COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/frontend/package.json ./apps/frontend/package.json

RUN corepack enable && pnpm install --frozen-lockfile

COPY apps/frontend ./apps/frontend

RUN pnpm --filter frontend build:embedded \
    && test -f apps/frontend/out/index.html \
    && test -f apps/frontend/out/404.html

FROM rust:1.94-bookworm AS backend-build

WORKDIR /workspace

COPY . ./
COPY --from=frontend-build /workspace/apps/frontend/out ./apps/frontend/out

RUN cargo build --locked --release --package backend --bin taco --features embedded-frontend

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates curl \
    && rm --recursive --force /var/lib/apt/lists/*

COPY --from=backend-build /workspace/target/release/taco /usr/local/bin/taco

ENV TACO_DATA_DIR=/data

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/taco"]
