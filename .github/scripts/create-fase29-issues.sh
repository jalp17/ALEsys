#!/bin/bash
# .github/scripts/create-fase29-issues.sh
# Creates GitHub issues from template files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEMPLATES_DIR="$PROJECT_ROOT/.github/ISSUE_TEMPLATES"

echo "Creating Fase 29/30 issues from templates..."

for file in "$TEMPLATES_DIR"/TICKET-29.*.md "$TEMPLATES_DIR"/TICKET-30.*.md; do
    if [ -f "$file" ]; then
        title=$(basename "$file" .md | sed 's/TICKET-/feat: /; s/-/ /g')
        gh issue create \
            --title "$title" \
            --body-file "$file" \
            --label "fase29,fase30" \
            || echo "Warning: Could not create issue from $file (may already exist)"
        echo "Created: $title"
    fi
done

echo "Done! All Fase 29/30 issues created."