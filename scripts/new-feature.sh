#!/usr/bin/env bash
# ALEsys - Script para crear nuevas features
# Uso: ./scripts/new-feature.sh <fase> <area> <descripcion>
# Ejemplo: ./scripts/new-feature.sh 1 core hybrid-search

set -euo pipefail

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Función de ayuda
show_help() {
    echo -e "${BLUE}ALEsys - Crear Nueva Feature${NC}"
    echo ""
    echo -e "${YELLOW}Uso:${NC}"
    echo "  ./scripts/new-feature.sh <fase> <area> <descripcion>"
    echo ""
    echo -e "${YELLOW}Ejemplos:${NC}"
    echo "  ./scripts/new-feature.sh 1 core hybrid-search"
    echo "  ./scripts/new-feature.sh 1 api chat-endpoint"
    echo "  ./scripts/new-feature.sh 1 webui chat-ui"
    echo "  ./scripts/new-feature.sh 2 core generation-engine"
    echo ""
    echo -e "${YELLOW}Áreas disponibles:${NC}"
    echo "  core   - Rust core logic (LLM, embeddings, grafos)"
    echo "  api    - API REST/WebSocket"
    echo "  webui  - Frontend React"
    echo "  php    - PHP backend (auth, proxy)"
    echo "  infra  - Infraestructura (Docker, CI/CD)"
    echo "  tauri  - Desktop wrapper"
    echo ""
    echo -e "${YELLOW}Fases:${NC}"
    echo "  1 - Chat con GraphRAG"
    echo "  2 - Generación de archivos"
    echo "  3 - Sesiones multi-usuario"
    echo "  4 - Optimización"
    echo "  5 - Visualización de grafos"
    echo "  6 - Búsqueda avanzada"
    echo "  7 - Sandbox de ejecución"
    echo "  8 - Tauri Desktop"
}

# Validar argumentos
if [[ $# -lt 3 ]] || [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    show_help
    exit 0
end

FASE="$1"
AREA="$2"
DESCRIPCION="$3"

# Validar fase
if ! [[ "$FASE" =~ ^[1-8]$ ]]; then
    echo -e "${RED}Error: Fase inválida. Debe ser un número del 1 al 8.${NC}"
    exit 1
fi

# Validar área
AREAS_VALIDAS=("core" "api" "webui" "php" "infra" "tauri")
if [[ ! " ${AREAS_VALIDAS[@]} " =~ " ${AREA} " ]]; then
    echo -e "${RED}Error: Área inválida.${NC}"
    echo "Áreas válidas: ${AREAS_VALIDAS[*]}"
    exit 1
fi

# Validar descripción
if [[ ! "$DESCRIPCION" =~ ^[a-z0-9-]+$ ]]; then
    echo -e "${RED}Error: La descripción debe ser en minúsculas, sin espacios, usando guiones.${NC}"
    echo "Ejemplo: hybrid-search, chat-endpoint, session-manager"
    exit 1
fi

# Construir nombre de rama
BRANCH_NAME="feature/${FASE}-${AREA}-${DESCRIPCION}"

# Verificar si estamos en un repositorio git
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: No estás en un repositorio git.${NC}"
    exit 1
fi

# Verificar si hay cambios sin commit
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${YELLOW}Advertencia: Hay cambios sin commit en el directorio de trabajo.${NC}"
    echo -e "${YELLOW}¿Deseas continuar? (s/n)${NC}"
    read -r response
    if [[ "$response" != "s" ]] && [[ "$response" != "S" ]]; then
        echo -e "${RED}Operación cancelada.${NC}"
        exit 1
    fi
fi

# Obtener rama actual
CURRENT_BRANCH=$(git branch --show-current)

# Verificar si la rama de fase existe
PHASE_BRANCH="phase-${FASE}-*"
if ! git show-ref --verify --quiet "refs/heads/phase-${FASE}"*; then
    echo -e "${YELLOW}La rama de fase 'phase-${FASE}-*' no existe.${NC}"
    echo -e "${YELLOW}¿Deseas crearla desde main? (s/n)${NC}"
    read -r response
    if [[ "$response" == "s" ]] || [[ "$response" == "S" ]]; then
        git checkout main
        git pull origin main
        git checkout -b "phase-${FASE}-"
        echo -e "${GREEN}Rama de fase creada: phase-${FASE}-${NC}"
    else
        echo -e "${RED}Operación cancelada.${NC}"
        exit 1
    fi
fi

# Verificar si la rama ya existe
if git show-ref --verify --quiet "refs/heads/${BRANCH_NAME}"; then
    echo -e "${RED}Error: La rama '${BRANCH_NAME}' ya existe.${NC}"
    exit 1
fi

# Crear rama de feature desde la rama de fase
echo -e "${BLUE}Creando rama de feature desde phase-${FASE}...${NC}"
git checkout "phase-${FASE}"
git pull origin "phase-${FASE}" 2>/dev/null || true
git checkout -b "${BRANCH_NAME}"

# Crear estructura de directorios básica
echo -e "${BLUE}Creando estructura de directorios...${NC}"

case "$AREA" in
    core)
        mkdir -p "crates/core/src"
        ;;
    api)
        mkdir -p "crates/api/src"
        ;;
    webui)
        mkdir -p "webui/src"
        ;;
    php)
        mkdir -p "server/includes"
        ;;
    infra)
        mkdir -p "docker"
        mkdir -p "scripts"
        ;;
    tauri)
        mkdir -p "desktop/src"
        ;;
esac

# Crear archivo de notas para la feature
mkdir -p ".features"
cat > ".features/${BRANCH_NAME}.md" << EOF
# Feature: ${BRANCH_NAME}

## Información
- **Fase:** ${FASE}
- **Área:** ${AREA}
- **Descripción:** ${DESCRIPCION}
- **Fecha de creación:** $(date +"%Y-%m-%d %H:%M:%S")
- **Estado:** En desarrollo

## Objetivos
- [ ] Implementar funcionalidad principal
- [ ] Escribir tests unitarios
- [ ] Documentar cambios
- [ ] Actualizar AGENT.md

## Dependencias
- [ ]

## Notas
-

## Checklist de Merge
- [ ] Código compilando sin warnings
- [ ] Tests unitarios pasando
- [ ] Tests de integración pasando
- [ ] Documentación actualizada
- [ ] Variables de entorno documentadas
EOF

echo ""
echo -e "${GREEN}=========================================="
echo -e "Feature creada exitosamente!"
echo -e "=========================================="
echo ""
echo -e "${BLUE}Rama:${NC} ${BRANCH_NAME}"
echo -e "${BLUE}Desde:${NC} phase-${FASE}"
echo -e "${BLUE}Área:${NC} ${AREA}"
echo -e "${BLUE}Descripción:${NC} ${DESCRIPCION}"
echo ""
echo -e "${YELLOW}Próximos pasos:${NC}"
echo "1. Desarrollar la feature"
echo "2. Crear commits atómicos:"
echo "   git commit -m \"feat(${AREA}): implement ${DESCRIPCION}\""
echo "3. Cuando esté completa, mergear a la fase:"
echo "   git checkout phase-${FASE}"
echo "   git merge ${BRANCH_NAME}"
echo "4. Cuando TODAS las features de la fase estén completas:"
echo "   git checkout main"
echo "   git merge phase-${FASE}"
echo "   git tag -a v${FASE}.0.0 -m \"Release v${FASE}.0.0: Fase ${FASE}\""
echo ""
echo -e "${GREEN}¡Buena suerte con el desarrollo!${NC}"