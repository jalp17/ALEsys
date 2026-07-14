# Instalación

## Requisitos previos
- Python 3.10+
- PostgreSQL 14+ con extensión `pgvector`
- Variables de entorno en `.env` (ver `.env.example`)

## Pasos

```bash
python -m pip install --upgrade pip
pip install -r requirements.txt
cp .env.example .env
```

Configurá las variables del `.env` según tu entorno.

### Base de datos
Inicializá las tablas antes de indexar:

```bash
python -m cli db-init
```

### Verificación rápida
Ejecutá la validación de entorno:

```bash
bash check-env.sh
```

## Solución de problemas
- Confirmá que el puerto de PostgreSQL esté accesible desde el host.
- Si se corta la indexación, reanudá con `python pipeline.py`; el sistema vuelve a procesar los mismos archivos desde cero (sin estado intermedio).
