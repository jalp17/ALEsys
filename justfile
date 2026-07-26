# ALEsys Justfile
# Run `just --list` to see all commands
# Install just: cargo install just

# ── Default ────────────────────────────────────────────────────────

default:
    @just --list

# ── Build ──────────────────────────────────────────────────────────

# Build all Rust crates (debug)
build:
    cargo build --workspace

# Build all Rust crates (release)
build-release:
    cargo build --release --workspace

# Build only the CLI binary
build-cli:
    cargo build -p alesys-cli

# Build only the API binary
build-api:
    cargo build -p alesys-api

# Build only the WebUI
build-web:
    cd webui && npm run build:web

# Build everything (Rust + WebUI)
build-all:
    cargo build --release --workspace
    cd webui && npm run build:web

# ── Development ────────────────────────────────────────────────────

# Start API server in dev mode
serve:
    cargo run --bin alesys-api

# Start WebUI dev server (Vite)
web:
    cd webui && npm run dev

# Start both API + WebUI in dev mode (parallel)
dev: _check-deps
    @echo "Starting API on :3000 and WebUI on :5173..."
    @cargo run --bin alesys-api &
    @cd webui && npm run dev

# Run initial dev setup
dev-setup:
    bash scripts/setup-dev.sh

# ── Testing ────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test --workspace

# Run tests for CLI only
test-cli:
    cargo test -p alesys-cli

# Run tests for core only
test-core:
    cargo test -p alesys-core

# Run tests for API only
test-api:
    cargo test -p alesys-api

# Run tests without default features (faster, no LLM)
test-fast:
    cargo test --workspace --no-default-features

# ── Linting & Formatting ──────────────────────────────────────────

# Run clippy linter
lint:
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Check formatting (dry-run)
fmt-check:
    cargo fmt --all -- --check

# Lint + format check
check: fmt-check lint

# ── Docker ─────────────────────────────────────────────────────────

# Start all Docker services
docker-up:
    docker compose -f docker/docker-compose.yml up -d

# Start services and rebuild images
docker-build:
    docker compose -f docker/docker-compose.yml up -d --build

# Stop all Docker services
docker-down:
    docker compose -f docker/docker-compose.yml down

# Stop services and remove volumes
docker-clean:
    docker compose -f docker/docker-compose.yml down -v

# Show Docker service status
docker-ps:
    docker compose -f docker/docker-compose.yml ps

# Show Docker service logs
docker-logs service="":
    docker compose -f docker/docker-compose.yml logs {{ if service == "" { "" } else { service } }} -f --tail=50

# Restart a specific service
docker-restart service:
    docker compose -f docker/docker-compose.yml restart {{ service }}

# ── Database ───────────────────────────────────────────────────────

# Initialize database (create tables)
db-init:
    cargo run --bin alesys -- db init

# Drop all database tables
db-drop:
    cargo run --bin alesys -- db drop

# Drop tables without confirmation
db-drop-force:
    cargo run --bin alesys -- db drop --force

# Run pending migrations
db-migrate:
    cargo run --bin alesys -- db migrate

# Show migration status
db-migrate-status:
    cargo run --bin alesys -- db migrate-status

# Create database backup
db-backup output="./backups":
    cargo run --bin alesys -- db backup --output {{ output }}

# ── Sessions ───────────────────────────────────────────────────────

# Create a new session
session-new name="":
    cargo run --bin alesys -- session new {{ if name == "" { "" } else { "--name " + name } }}

# List active sessions
session-list:
    cargo run --bin alesys -- session list

# Close a session
session-close session_id:
    cargo run --bin alesys -- session close {{ session_id }}

# ── LLM ────────────────────────────────────────────────────────────

# Show LLM status
llm-status:
    cargo run --bin alesys -- llm status

# Load LLM model into memory
llm-load:
    cargo run --bin alesys -- llm load

# Unload LLM model (free RAM)
llm-unload:
    cargo run --bin alesys -- llm unload

# ── Search & Query ─────────────────────────────────────────────────

# Search indexed documents
search query mode="hybrid":
    cargo run --bin alesys -- search "{{ query }}" --mode {{ mode }}

# List indexed documents
list:
    cargo run --bin alesys -- list

# Chat with RAG context
ask question:
    cargo run --bin alesys -- ask "{{ question }}"

# ── Graph ──────────────────────────────────────────────────────────

# Show graph statistics
graph-stats:
    cargo run --bin alesys -- graph stats

# Export graph to JSON
graph-export output="graph-export.json":
    cargo run --bin alesys -- graph export --output {{ output }}

# ── System ─────────────────────────────────────────────────────────

# Show system status
status:
    cargo run --bin alesys -- status

# Show current configuration
config:
    cargo run --bin alesys -- config

# ── Install ────────────────────────────────────────────────────────

# Install CLI binary to ~/.cargo/bin
install:
    cargo install --path crates/cli

# Install release build
install-release:
    cargo install --path crates/cli --release

# ── Production ─────────────────────────────────────────────────────

# Full production deploy (build + test + docker)
deploy: build-release test-fast
    docker compose -f docker/docker-compose.yml up -d --build
    @echo "Waiting for services..."
    @sleep 10
    @curl -sf http://localhost:3000/health && echo "Health check OK" || echo "Health check FAILED"

# ── Clean ──────────────────────────────────────────────────────────

# Remove all build artifacts
clean:
    cargo clean
    rm -rf webui/dist

# Remove build artifacts + Docker volumes
clean-all: clean docker-clean

# ── Internal ───────────────────────────────────────────────────────

# Check that required tools are installed
_check-deps:
    @command -v cargo >/dev/null 2>&1 || (echo "cargo not found. Install Rust: https://rustup.rs" && exit 1)
    @command -v node >/dev/null 2>&1 || (echo "node not found. Install: https://nodejs.org" && exit 1)
    @command -v npm >/dev/null 2>&1 || (echo "npm not found" && exit 1)
