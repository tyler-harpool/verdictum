#!/bin/bash

# Script to fix all RepositoryFactory calls to handle Results

echo "Fixing all repository factory calls..."

# Fix attorney handler
echo "Fixing attorney.rs..."
sed -i '' 's/RepositoryFactory::attorney_repo(&req);/RepositoryFactory::attorney_repo(\&req)?;/g' src/handlers/attorney.rs

# Fix case handler
echo "Fixing criminal_case.rs..."
sed -i '' 's/RepositoryFactory::case_repo(&req);/RepositoryFactory::case_repo(\&req)?;/g' src/handlers/criminal_case.rs

# Fix config handler
echo "Fixing config.rs..."
sed -i '' 's/RepositoryFactory::config_repo(&req);/RepositoryFactory::config_repo(\&req)?;/g' src/handlers/config.rs

# Fix deadline handler
echo "Fixing deadline.rs..."
sed -i '' 's/RepositoryFactory::deadline_repo(&req);/RepositoryFactory::deadline_repo(\&req)?;/g' src/handlers/deadline.rs

# Fix docket handler (already done but just in case)
echo "Fixing docket.rs..."
sed -i '' 's/RepositoryFactory::docket_repo(&req);/RepositoryFactory::docket_repo(\&req)?;/g' src/handlers/docket.rs

# Fix judge handler
echo "Fixing judge.rs..."
sed -i '' 's/RepositoryFactory::judge_repo(&req);/RepositoryFactory::judge_repo(\&req)?;/g' src/handlers/judge.rs

# Fix opinion handler
echo "Fixing opinion.rs..."
sed -i '' 's/RepositoryFactory::document_repo(&req);/RepositoryFactory::document_repo(\&req)?;/g' src/handlers/opinion.rs

# Fix order handler
echo "Fixing order.rs..."
sed -i '' 's/RepositoryFactory::document_repo(&req);/RepositoryFactory::document_repo(\&req)?;/g' src/handlers/order.rs

# Fix sentencing handler
echo "Fixing sentencing.rs..."
sed -i '' 's/RepositoryFactory::sentencing_repo(&req);/RepositoryFactory::sentencing_repo(\&req)?;/g' src/handlers/sentencing.rs

# Fix admin handler
echo "Fixing admin.rs..."
sed -i '' 's/RepositoryFactory::config_repo(&req);/RepositoryFactory::config_repo(\&req)?;/g' src/handlers/admin.rs

echo "Done!"