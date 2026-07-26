#!/usr/bin/env bash
set -euo pipefail

# setup-mineru.sh
# Configura el entorno Python e instala MinerU para el pipeline de ingesta.

echo "=== ALEsys - Setup MinerU ==="

PYTHON_VERSION="${PYTHON_VERSION:-3.12}"
VENV_DIR="${VENV_DIR:-/opt/mineru}"
MODELS_DIR="${MODELS_DIR:-/opt/mineru/models}"

echo "Python version: $PYTHON_VERSION"
echo "Venv dir: $VENV_DIR"
echo "Models dir: $MODELS_DIR"

# 1. Crear virtualenv
python3 -m venv "$VENV_DIR"
source "$VENV_DIR/bin/activate"

# 2. Instalar MinerU con soporte GPU
pip install --upgrade pip
pip install "magic-pdf[gpu]"

# 3. Descargar modelos
echo "Descargando modelos..."
python -m magic_pdf --download-models

# 4. Verificar instalación
echo "Verificando..."
magic-pdf --version
nvidia-smi || echo "WARNING: nvidia-smi no disponible, MinerU usará CPU"

echo "=== Setup completado ==="
echo "Activar venv: source $VENV_DIR/bin/activate"
