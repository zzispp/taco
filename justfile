test_timeout_seconds := "60"

list:
    just --list

build:
    cargo build

build-release:
    pnpm --filter frontend build:embedded
    cargo build --locked --release -p backend --bin taco --features embedded-frontend

check:
    cargo check

format:
    cargo fmt -q --all

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    perl -e 'my $timeout = shift; my $pid = fork(); die "fork failed: $!" unless defined $pid; if ($pid == 0) { exec @ARGV or die "exec failed: $!"; } $SIG{ALRM} = sub { kill "TERM", $pid; exit 124; }; alarm $timeout; waitpid($pid, 0); exit($? >> 8);' {{test_timeout_seconds}} cargo test

quality-precommit:
    scripts/quality/ensure-rust-quality-tools.sh precommit
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo check --workspace --all-targets
    just test

quality-complete: quality-precommit
    scripts/quality/ensure-rust-quality-tools.sh complete
    cargo audit
    cargo deny check

install-git-hooks:
    mkdir -p .git/hooks
    cp scripts/git-hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

run-backend:
    cargo run -p backend --bin taco --

backend-migration ARGS:
    cargo run -p backend --bin taco -- migration {{ARGS}}

services-up:
    COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose up -d

services-down:
    COMPOSE_DISABLE_ENV_FILE=1 COMPOSE_ENV_FILES= docker compose down
