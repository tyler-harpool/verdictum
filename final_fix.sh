#!/bin/bash

# Add missing json imports where needed
echo "Adding missing json imports..."

# For criminal_case.rs
sed -i '' '9a\
use crate::utils::json_response as json;' src/handlers/criminal_case.rs

# For deadline.rs
sed -i '' '/use crate::utils::repository_factory/a\
use crate::utils::json_response as json;' src/handlers/deadline.rs

# For judge.rs
sed -i '' '/use crate::utils::repository_factory/a\
use crate::utils::json_response as json;' src/handlers/judge.rs

# For sentencing.rs
sed -i '' '/use crate::utils::repository_factory/a\
use crate::utils::json_response as json;' src/handlers/sentencing.rs

echo "Done!"