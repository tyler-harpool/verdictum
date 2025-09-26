#!/bin/bash

# Script to update RepositoryFactory calls to handle Results

echo "Updating docket handler..."
sed -i '' 's/let repo = RepositoryFactory::docket_repo(&req);/let repo = RepositoryFactory::docket_repo(\&req)?;/g' src/handlers/docket.rs

echo "Updating deadline handler..."
sed -i '' 's/let repo = RepositoryFactory::deadline_repo(&req);/let repo = RepositoryFactory::deadline_repo(\&req)?;/g' src/handlers/deadline.rs

echo "Updating sentencing handler..."
sed -i '' 's/let repo = RepositoryFactory::sentencing_repo(&req);/let repo = RepositoryFactory::sentencing_repo(\&req)?;/g' src/handlers/sentencing.rs

echo "Updating judge handler..."
sed -i '' 's/let repo = RepositoryFactory::judge_repo(&req);/let repo = RepositoryFactory::judge_repo(\&req)?;/g' src/handlers/judge.rs

echo "Updating criminal_case handler..."
sed -i '' 's/let repo = RepositoryFactory::case_repo(&req);/let repo = RepositoryFactory::case_repo(\&req)?;/g' src/handlers/criminal_case.rs
sed -i '' 's/let case_repo = RepositoryFactory::case_repo(&req);/let case_repo = RepositoryFactory::case_repo(\&req)?;/g' src/handlers/criminal_case.rs

echo "Updating attorney handler..."
sed -i '' 's/let repo = RepositoryFactory::attorney_repo(&req);/let repo = RepositoryFactory::attorney_repo(\&req)?;/g' src/handlers/attorney.rs

echo "Updating opinion handler..."
sed -i '' 's/let repo = RepositoryFactory::document_repo(&req);/let repo = RepositoryFactory::document_repo(\&req)?;/g' src/handlers/opinion.rs

echo "Updating order handler..."
sed -i '' 's/let repo = RepositoryFactory::document_repo(&req);/let repo = RepositoryFactory::document_repo(\&req)?;/g' src/handlers/order.rs

echo "Updating config handler..."
sed -i '' 's/let repo = RepositoryFactory::config_repo(&req);/let repo = RepositoryFactory::config_repo(\&req)?;/g' src/handlers/config.rs

echo "Done!"