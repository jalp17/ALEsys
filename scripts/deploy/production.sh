#!/bin/bash
# 🚀🐱 ALEsys Production Deployment Script
# Usage: ./scripts/deploy/production.sh [environment]

set -e

ENVIRONMENT=${1:-production}
VERSION="2.0.0"

echo "🚀🐱 ALEsys Production Deployment v${VERSION}"
echo "Environment: ${ENVIRONMENT}"
echo "=========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    command -v cargo >/dev/null 2>&1 || { log_error "Rust not installed"; exit 1; }
    command -v pnpm >/dev/null 2>&1 || { log_error "pnpm not installed"; exit 1; }
    command -v docker >/dev/null 2>&1 || { log_error "Docker not installed"; exit 1; }
    
    log_info "All prerequisites met"
}

build_backend() {
    log_info "Building backend (release mode)..."
    cargo build --release --workspace
    
    log_info "Backend built successfully"
}

build_frontend() {
    log_info "Building frontend..."
    cd webui
    pnpm install
    pnpm build
    cd ..
    
    log_info "Frontend built successfully"
}

run_tests() {
    log_info "Running test suite..."
    cargo test --workspace --no-default-features --features test -- --test-threads=1
    
    log_info "All tests passed"
}

deploy_docker() {
    log_info "Deploying with Docker Compose..."
    
    docker compose -f docker/docker-compose.yml up -d --build
    
    log_info "Docker containers deployed"
}

health_check() {
    log_info "Waiting for services to start..."
    sleep 10
    
    log_info "Running health check..."
    curl -f http://localhost:3000/health || { log_error "Health check failed"; exit 1; }
    
    log_info "Health check passed"
}

backup_database() {
    log_info "Creating database backup..."
    
    BACKUP_DIR="./backups"
    mkdir -p ${BACKUP_DIR}
    
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    docker exec alesys-postgres pg_dump -U alesys alesys > ${BACKUP_DIR}/backup_${TIMESTAMP}.sql
    
    log_info "Database backed up to ${BACKUP_DIR}/backup_${TIMESTAMP}.sql"
}

# Main deployment flow
main() {
    check_prerequisites
    run_tests
    build_backend
    build_frontend
    
    if [ "${ENVIRONMENT}" = "production" ]; then
        backup_database
    fi
    
    deploy_docker
    health_check
    
    echo ""
    echo "=========================================="
    log_info "Deployment completed successfully!"
    echo "Version: ${VERSION}"
    echo "Environment: ${ENVIRONMENT}"
    echo "=========================================="
}

main