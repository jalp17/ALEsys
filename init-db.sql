-- ALEsys - Inicialización de base de datos
-- Se ejecuta automáticamente al crear el contenedor PostgreSQL

-- Habilitar extensión pgvector
CREATE EXTENSION IF NOT EXISTS vector;

-- Crear esquema si no existe (opcional, usa public por defecto)
-- CREATE SCHEMA IF NOT EXISTS alesys;
-- SET search_path TO alesys, public;

-- Configurar permisos
GRANT ALL PRIVILEGES ON DATABASE alesys TO alesys;
GRANT ALL ON SCHEMA public TO alesys;

-- Verificar instalación
SELECT extname, extversion FROM pg_extension WHERE extname = 'vector';