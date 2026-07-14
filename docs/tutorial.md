# Tutorial

1. Configurá `.env` con tus rutas y credenciales.
2. Iniciá la base de datos:
   ```bash
   python -m cli db-init
   ```
3. Ejecutá el pipeline:
   ```bash
   python pipeline.py
   ```
4. Probá una consulta:
   ```bash
   python test_queries.py "campo eléctrico"
   ```
5. Para respuestas generadas:
   ```bash
   python test_queries.py "ley de Faraday" --hybrid
   ```

Recomendación: usar carpetas locales pequeñas para pruebas rápidas y aumentar `BOOKS_DIR` solo cuando quieras indexar la colección completa.
