#!/bin/bash
# 🐱 ALEsys Development Setup Script

set -e

echo "🐱 Setting up ALEsys development environment..."

# Install Rust dependencies
echo "🐱 Installing Rust dependencies..."
cargo install --path crates/core

# Install Node.js dependencies
echo "🐱 Installing Node.js dependencies..."
cd webui
pnpm install
cd ..

# Setup database
echo "🐱 Setting up database..."
docker compose -f docker/docker-compose.yml up -d postgres

# Run migrations
echo "🐱 Running database migrations..."
cargo run --bin alesys-cli -- migrate up

echo "🐱 Development environment ready!"
echo "🐱 Run 'pnpm dev' to start"