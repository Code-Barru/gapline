#!/usr/bin/env bash
# Verifies that cli/src/cli/commands/ contains no forbidden core imports.
# Exit 1 on violation, 0 on success. Run from repo root.
set -euo pipefail

COMMANDS_DIR="cli/src/cli/commands"

FORBIDDEN=(
    "gapline_core::parser::FeedLoader"
    "gapline_core::parser::FeedSource"
    "gapline_core::integrity"
    "gapline_core::crud::update::validate_update"
    "gapline_core::crud::update::apply_update"
    "gapline_core::crud::delete::validate_delete"
    "gapline_core::crud::delete::apply_delete"
    "gapline_core::crud::delete::required_files"
    "gapline_core::crud::create::validate_create"
    "gapline_core::crud::create::apply_create"
    "gapline_core::crud::create::required_files"
    "gapline_core::crud::setters"
    "gapline_core::crud::common"
)

# rules.rs is the single documented exception (ValidationEngine introspection)
EXCEPTION_FILE="rules.rs"

FAILED=0

for pattern in "${FORBIDDEN[@]}"; do
    while IFS= read -r line; do
        filename=$(echo "$line" | cut -d: -f1)
        basename_file=$(basename "$filename")
        if [[ "$basename_file" == "$EXCEPTION_FILE" ]]; then
            continue
        fi
        echo "FORBIDDEN IMPORT in $line"
        FAILED=1
    done < <(grep -rn "$pattern" "$COMMANDS_DIR" 2>/dev/null || true)
done

if [[ $FAILED -ne 0 ]]; then
    echo "Audit FAILED: forbidden imports found in $COMMANDS_DIR"
    exit 1
fi

echo "Audit PASSED: no forbidden imports in $COMMANDS_DIR"
