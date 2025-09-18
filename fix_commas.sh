#!/bin/bash

# Fix missing commas after tag attributes in all handler files
for file in src/handlers/attorney.rs src/handlers/deadline.rs src/handlers/docket.rs src/handlers/judge.rs src/handlers/opinion.rs src/handlers/order.rs src/handlers/sentencing.rs; do
    echo "Fixing $file..."
    # Add comma after tag = "..." when followed by params(
    sed -i '' 's/tag = "\([^"]*\)"$/tag = "\1",/g' "$file"
done

echo "All files fixed!"