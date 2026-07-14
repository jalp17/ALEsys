# Guía de uso

## Requisitos
- PostgreSQL con pgvector
- Variables de entorno en `.env` (ver `.env.example`)

## Instalación
```bash
python -m pip install --upgrade pip
pip install -r requirements.txt
```

## Inicializar la base de datos
```bash
python -m cli db-init
```

## Ejecutar el pipeline
```bash
python pipeline.py
```

## Consultas
```bash
python test_queries.py "ley de Faraday"
python test_queries.py "campo eléctrico" --graph Faraday
python test_queries.py "inducción electromagnética" --hybrid
```

## Configuración recomendada
- Usar `.env` en la raíz del proyecto.
- No exponer `OPENROUTER_API_KEY` en el repositorio.
- Verificar `BOOKS_DIR` apunta a la carpeta con archivos `.md`.
