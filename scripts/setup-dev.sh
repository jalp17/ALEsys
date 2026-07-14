#!/bin/bash
set -e

echo "🚀 Setup inicial de ALEsys"
echo "=========================="
echo ""

# === Verificar dependencias ===
echo "📦 Verificando dependencias..."

check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 no instalado. $2"
        exit 1
    fi
    echo "✅ $1 instalado"
}

check_command rustc "Instala desde https://rustup.rs"
check_command cargo "Instala desde https://rustup.rs"
check_command node "Instala desde https://nodejs.org"
check_command pnpm "Ejecuta: npm install -g pnpm"

if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker no disponible. Algunas features no funcionarán."
else
    echo "✅ docker instalado"
fi

# === Instalar Rust dependencies ===
echo ""
echo "🦀 Instalando Rust dependencies..."
rustup component add clippy rustfmt 2>/dev/null || true
echo "✅ Rust toolchain lista"

# === Instalar Node dependencies ===
echo ""
echo "📦 Instalando Node dependencies..."
if [ -f "webui/package.json" ]; then
    cd webui
    pnpm install
    cd ..
    echo "✅ WebUI dependencies instaladas"
else
    echo "⚠️  webui/package.json no encontrado"
fi

# === Instalar PHP dependencies ===
echo ""
echo "🐘 Instalando PHP dependencies..."
if [ -f "server/composer.json" ]; then
    cd server
    if command -v composer &> /dev/null; then
        composer install 2>/dev/null || echo "⚠️  Composer install falló, continuar manualmente"
    else
        echo "⚠️  Composer no instalado. Instala desde https://getcomposer.org"
    fi
    cd ..
else
    echo "⚠️  server/composer.json no encontrado"
fi

# === Setup databases ===
echo ""
echo "🗄️  Configurando databases..."
if command -v docker &> /dev/null; then
    echo "Iniciando PostgreSQL con Docker..."
    docker compose -f docker/docker-compose.yml up -d postgres-core postgres-users
    
    echo "⏳ Esperando a que PostgreSQL esté listo (30s)..."
    sleep 30
    
    # Verificar conexión
    if docker compose -f docker/docker-compose.yml exec -T postgres-core pg_isready -U alesys &>/dev/null; then
        echo "✅ PostgreSQL core listo"
    else
        echo "⚠️  PostgreSQL core no responde. Verificar logs."
    fi
    
    if docker compose -f docker/docker-compose.yml exec -T postgres-users pg_isready -U alesys &>/dev/null; then
        echo "✅ PostgreSQL users listo"
    else
        echo "⚠️  PostgreSQL users no responde. Verificar logs."
    fi
else
    echo "⚠️  Docker no disponible. Configura PostgreSQL manualmente:"
    echo "   1. Crea database 'alesys' con extensión pgvector"
    echo "   2. Crea database 'alesys_users'"
    echo "   3. Ejecuta docker/init-db.sql"
fi

# === Build inicial ===
echo ""
echo "🔨 Build inicial..."
if cargo build --workspace 2>/dev/null; then
    echo "✅ Rust build exitoso"
else
    echo "⚠️  Rust build falló. Ejecutar: cargo build --workspace"
fi

if [ -f "webui/package.json" ]; then
    cd webui
    if pnpm build:web 2>/dev/null; then
        echo "✅ WebUI build exitoso"
    else
        echo "⚠️  WebUI build falló. Ejecutar: cd webui && pnpm build:web"
    fi
    cd ..
fi

# === Verificar .env ===
echo ""
echo "📝 Verificando configuración..."
if [ ! -f ".env" ]; then
    if [ -f "docker/.env.example" ]; then
        echo "Creando .env desde .env.example..."
        cp docker/.env.example .env
        echo "⚠️  IMPORTANTE: Edita .env con tus configuraciones"
    else
        echo "⚠️  docker/.env.example no encontrado. Crea .env manualmente"
    fi
else
    echo "✅ .env ya existe"
fi

# === Resumen final ===
echo ""
echo "================================"
echo "✅ Setup completado!"
echo "================================"
echo ""
echo "Próximos pasos:"
echo "  1. Edita .env con tus configuraciones"
echo "  2. Ejecuta: docker compose -f docker/docker-compose.yml up -d"
echo "  3. Ejecuta: pnpm dev"
echo "  4. Abre http://localhost:5173 (dev) o http://localhost:8080 (PHP)"
echo ""
echo "Documentación:"
echo "  - README.md: Guía principal"
echo "  - docs/: Documentación detallada"
echo ""