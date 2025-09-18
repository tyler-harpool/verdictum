#!/bin/bash

# Script to add X-Court-District header to all utoipa paths that don't have it

add_header_to_file() {
    local file=$1
    echo "Processing $file..."

    # Create a temporary file
    temp_file="${file}.tmp"

    # Process the file
    awk '
    /^#\[utoipa::path\(/ {
        in_path = 1
        path_block = $0 "\n"
        next
    }
    in_path {
        path_block = path_block $0 "\n"
        if (/^\)\]$/) {
            # Check if X-Court-District already exists
            if (path_block !~ /X-Court-District/) {
                # Check if params section exists
                if (path_block ~ /params\(/) {
                    # Add X-Court-District as first param after params(
                    gsub(/params\(/, "params(\n        (\"X-Court-District\" = String, Header, description = \"Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)\", example = \"SDNY\"),", path_block)
                } else {
                    # Add params section before closing
                    gsub(/\)\]/, "    params(\n        (\"X-Court-District\" = String, Header, description = \"Federal court district (e.g., SDNY, EDNY, NDCA, CDCA)\", example = \"SDNY\")\n    ),\n)]", path_block)
                }
            }
            printf "%s", path_block
            in_path = 0
            path_block = ""
            next
        }
        next
    }
    { print }
    ' "$file" > "$temp_file"

    # Replace original file
    mv "$temp_file" "$file"
    echo "✓ Updated $file"
}

# Process each handler file that uses RepositoryFactory
for file in attorney.rs deadline.rs docket.rs judge.rs opinion.rs order.rs sentencing.rs; do
    add_header_to_file "/Users/THarpool/Code/personel/spin-todo-api/src/handlers/$file"
done

echo "Done! All files updated."