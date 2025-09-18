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
    spin_kv_deadline_repository::SpinKvDeadlineRepository,
    spin_kv_docket_repository::SpinKvDocketRepository,
    spin_kv_document_repository::SpinKvDocumentRepository,
    spin_kv_judge_repository::SpinKvJudgeRepository,
    spin_kv_sentencing_repository::SpinKvSentencingRepository,
};
use crate::utils::tenant;
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

    // Note: Additional helper methods for tenant access control can be added here
    // For now, tenant isolation is handled at the store level
}