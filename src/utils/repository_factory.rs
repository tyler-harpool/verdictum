//! Repository factory for multi-tenant support in federal court system
//!
//! This module implements a factory pattern for creating tenant-specific repository instances.
//! It centralizes tenant detection and repository creation, ensuring data isolation between
//! different federal court districts.
//!
//! # Architecture
//!
//! The RepositoryFactory acts as a single point of control for multi-tenant data access:
//!
//! ```text
//! HTTP Request
//!     |
//!     v
//! RepositoryFactory::attorney_repo(&req)
//!     |
//!     ├─> Extract tenant ID from:
//!     │   - X-Tenant-ID header
//!     │   - X-Court-District header
//!     │   - Subdomain (e.g., sdny.lexodus.gov)
//!     │   - Query parameter (?tenant=sdny)
//!     │
//!     ├─> Generate store name: "tenant_{district}"
//!     │
//!     └─> Return SpinKvAttorneyRepository::with_store(store_name)
//! ```
//!
//! # Multi-Tenant Support
//!
//! Supports all 94 federal district courts:
//! - SDNY (Southern District of New York)
//! - EDNY (Eastern District of New York)
//! - CDCA (Central District of California)
//! - ... and 91 more
//!
//! # Usage
//!
//! ```rust
//! use crate::utils::repository_factory::RepositoryFactory;
//!
//! pub fn create_attorney(req: Request, _params: Params) -> Response {
//!     // Automatically gets the correct tenant-specific repository
//!     let repo = RepositoryFactory::attorney_repo(&req);
//!     // All operations now scoped to the tenant
//!     repo.save_attorney(attorney)?;
//! }
//! ```
//!
//! # Security
//!
//! - Tenant IDs are sanitized to prevent injection attacks
//! - Each tenant's data is completely isolated
//! - Access control can be enforced via check_access()

use crate::adapters::{
    spin_kv_attorney_repository::SpinKvAttorneyRepository,
    spin_kv_case_repository::SpinKvCaseRepository,
    spin_kv_config_repository::SpinKvConfigRepository,
    spin_kv_deadline_repository::SpinKvDeadlineRepository,
    spin_kv_docket_repository::SpinKvDocketRepository,
    spin_kv_document_repository::SpinKvDocumentRepository,
    spin_kv_judge_repository::SpinKvJudgeRepository,
    spin_kv_sentencing_repository::SpinKvSentencingRepository,
    unified_config_feature_repository::UnifiedConfigFeatureRepository,
};
use crate::ports::feature_repository::FeatureRepository;
use std::sync::Arc;
use crate::utils::{tenant, url_tenant};
use spin_sdk::http::Request;

/// Factory for creating tenant-specific repositories.
///
/// This struct provides static methods to create repository instances
/// that are automatically scoped to the correct tenant based on the
/// incoming HTTP request.
pub struct RepositoryFactory;

impl RepositoryFactory {
    /// Creates a tenant-specific attorney repository.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request containing tenant identification
    ///
    /// # Returns
    ///
    /// A `SpinKvAttorneyRepository` instance scoped to the identified tenant
    ///
    /// # Example
    ///
    /// ```
    /// let repo = RepositoryFactory::attorney_repo(&req);
    /// let attorney = repo.find_attorney_by_id("123")?;
    /// ```
    pub fn attorney_repo(req: &Request) -> SpinKvAttorneyRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvAttorneyRepository::with_store(store_name)
    }

    /// Creates attorney repository with URL-based tenant extraction
    ///
    /// Extracts tenant from URL path (e.g., /api/courts/sdny/attorneys)
    /// Falls back to header-based extraction for backward compatibility
    pub fn attorney_repo_from_url(req: &Request) -> Result<SpinKvAttorneyRepository, String> {
        let tenant_id = url_tenant::get_tenant_from_request(req)?;
        let store_name = tenant::get_store_name(&tenant_id);
        Ok(SpinKvAttorneyRepository::with_store(store_name))
    }

    /// Creates a tenant-specific criminal case repository.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request containing tenant identification
    ///
    /// # Returns
    ///
    /// A `SpinKvCaseRepository` instance scoped to the identified tenant
    pub fn case_repo(req: &Request) -> SpinKvCaseRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvCaseRepository::with_store(store_name)
    }

    /// Creates case repository with URL-based tenant extraction
    pub fn case_repo_from_url(req: &Request) -> Result<SpinKvCaseRepository, String> {
        let tenant_id = url_tenant::get_tenant_from_request(req)?;
        let store_name = tenant::get_store_name(&tenant_id);
        Ok(SpinKvCaseRepository::with_store(store_name))
    }

    /// Get tenant-specific deadline repository
    pub fn deadline_repo(req: &Request) -> SpinKvDeadlineRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvDeadlineRepository::with_store(store_name)
    }

    /// Get tenant-specific docket repository
    pub fn docket_repo(req: &Request) -> SpinKvDocketRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvDocketRepository::with_store(store_name)
    }

    /// Get tenant-specific document repository
    pub fn document_repo(req: &Request) -> SpinKvDocumentRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvDocumentRepository::with_store(store_name)
    }

    /// Get tenant-specific judge repository
    pub fn judge_repo(req: &Request) -> SpinKvJudgeRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvJudgeRepository::with_store(store_name)
    }

    /// Get tenant-specific sentencing repository
    pub fn sentencing_repo(req: &Request) -> SpinKvSentencingRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);
        SpinKvSentencingRepository::with_store(store_name)
    }

    /// Creates a tenant-specific configuration repository.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request containing tenant identification
    ///
    /// # Returns
    ///
    /// A `SpinKvConfigRepository` instance scoped to the identified tenant
    ///
    /// # Example
    ///
    /// ```
    /// let repo = RepositoryFactory::config_repo(&req);
    /// let config = repo.get_merged_config("SDNY", Some("judge-123")).await?;
    /// ```
    pub fn config_repo(req: &Request) -> SpinKvConfigRepository {
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);

        // Determine court type from tenant ID or headers
        let court_type = Self::determine_court_type(req, &tenant_id);

        SpinKvConfigRepository::with_district(store_name, tenant_id.clone(), court_type)
    }

    /// Creates config repository with URL-based tenant extraction
    pub fn config_repo_from_url(req: &Request) -> Result<SpinKvConfigRepository, String> {
        let tenant_id = url_tenant::get_tenant_from_request(req)?;
        let store_name = tenant::get_store_name(&tenant_id);

        // Determine court type from URL or headers
        let court_type = Self::determine_court_type(req, &tenant_id);

        Ok(SpinKvConfigRepository::with_district(store_name, tenant_id.clone(), court_type))
    }

    /// Creates a tenant-specific feature repository.
    ///
    /// This returns a unified repository that bridges features to the config system.
    ///
    /// # Arguments
    ///
    /// * `req` - The HTTP request containing tenant identification
    ///
    /// # Returns
    ///
    /// A boxed `FeatureRepository` trait object
    pub fn feature_repo(req: &Request) -> Box<dyn FeatureRepository> {
        let config_repo = Arc::new(Self::config_repo(req));
        let tenant_id = tenant::get_tenant_id(req);
        let store_name = tenant::get_store_name(&tenant_id);

        Box::new(UnifiedConfigFeatureRepository::new(
            config_repo,
            store_name,
        ))
    }

    /// Determine the court type from the request or tenant ID
    fn determine_court_type(req: &Request, tenant_id: &str) -> String {
        // First check URL path for court type
        if let Some(court_type) = url_tenant::extract_court_type_from_path(&req.path()) {
            return court_type;
        }

        // Check for explicit court type header
        for (name, value) in req.headers() {
            if name == "x-court-type" {
                if let Ok(ct) = std::str::from_utf8(value.as_ref()) {
                    return ct.to_lowercase();
                }
            }
        }

        // Infer from tenant ID patterns
        match tenant_id {
            id if id.contains("bk") || id.contains("bankruptcy") => "bankruptcy".to_string(),
            id if id.ends_with("ca") => "appellate".to_string(),  // Circuit Appeals
            id if id == "scotus" => "supreme".to_string(),
            id if id.contains("tax") => "tax".to_string(),
            id if id.contains("trade") => "trade".to_string(),
            id if id.contains("claims") => "claims".to_string(),
            id if id.contains("fisa") => "fisa".to_string(),
            id if id.contains("ptab") => "patent".to_string(),
            id if id.contains("ttab") => "trademark".to_string(),
            id if id.contains("itc") => "itc".to_string(),
            id if id.contains("mspb") => "merit".to_string(),
            _ => "district".to_string(),  // Default to district court
        }
    }
}