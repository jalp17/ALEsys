# ALEsys - Dockerfile multi-stage
# Build: docker build -t alesys .
# Run: docker run --rm -it --network host -v $(pwd)/.env:/app/.env alesys

# ============================================================
# STAGE 1: Builder - Instalar dependencias y compilar
# ============================================================
FROM python:3.11-slim AS builder

# Instalar dependencias del sistema para compilar
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no-root
RUN useradd --create-home --shell /bin/bash appuser

WORKDIR /app

# Copiar solo requirements para cache de capa
COPY requirements.txt .

# Instalar dependencias Python en directorio de usuario
RUN pip install --no-cache-dir --user -r requirements.txt

# ============================================================
# STAGE 2: Runtime - Imagen final ligera
# ============================================================
FROM python:3.11-slim AS runtime

# Instalar solo runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Crear usuario no-root
RUN useradd --create-home --shell /bin/bash appuser

WORKDIR /app

# Copiar dependencias instaladas desde builder
COPY --from=builder /root/.local /home/appuser/.local

# Copiar código de la aplicación
COPY --chown=appuser:appuser . .

# Cambiar a usuario no-root
USER appuser

# Añadir .local al PATH
ENV PATH=/home/appuser/.local/bin:$PATH

# Variables de entorno por defecto
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PGHOST=localhost \
    PGPORT=5432 \
    PGUSER=alesys \
    PGPASSWORD=alesys \
    PGDATABASE=alesys

# Healthcheck
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pg_isready -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE || exit 1

# Entrypoint por defecto
ENTRYPOINT ["python", "main.py"]
CMD ["--help"]

# Labels
LABEL org.opencontainers.image.title="ALEsys" \
      org.opencontainers.image.description="GraphRAG-PG: Pipeline de ingesta híbrida sobre PostgreSQL" \
      org.opencontainers.image.source="https://github.com/jalp17/ALEsys" \
      org.opencontainers.image.licenses="MIT"