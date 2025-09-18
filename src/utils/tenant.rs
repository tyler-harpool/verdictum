//! Tenant management utilities for multi-tenant support
//!
//! This module handles tenant identification and store selection
//! based on court district or other tenant identifiers.

use spin_sdk::http::Request;

/// Extract tenant identifier from request
///
/// The tenant ID can come from:
/// 1. X-Tenant-ID header (for explicit tenant selection)
/// 2. X-Court-District header (for court-based tenancy)
/// 3. Host header subdomain (e.g., sdny.lexodus.gov)
/// 4. Query parameter ?tenant=xxx
/// 5. Default to "default" if none specified
pub fn get_tenant_id(req: &Request) -> String {
    // Check for explicit tenant header
    if let Some(tenant_id) = req.header("x-tenant-id") {
        if let Some(value) = tenant_id.as_str() {
            if !value.is_empty() {
                return sanitize_tenant_id(value);
            }
        }
    }

    // Check for court district header
    if let Some(court) = req.header("x-court-district") {
        if let Some(value) = court.as_str() {
            if !value.is_empty() {
                return sanitize_tenant_id(value);
            }
        }
    }

    // Check host header for subdomain
    if let Some(host) = req.header("host") {
        if let Some(value) = host.as_str() {
            if let Some(subdomain) = extract_subdomain(value) {
                return sanitize_tenant_id(&subdomain);
            }
        }
    }

    // Check query parameter
    let query = req.query();
    for param in query.split('&') {
        if let Some(value) = param.strip_prefix("tenant=") {
            if !value.is_empty() {
                return sanitize_tenant_id(value);
            }
        }
    }

    // No tenant specified - this is an error for data operations
    // Return a special value that will cause store open to fail
    // This prevents accidental data mixing
    "TENANT_NOT_SPECIFIED".to_string()
}

/// Extract subdomain from host header
fn extract_subdomain(host: &str) -> Option<String> {
    // Remove port if present
    let host = host.split(':').next()?;

    // Split by dots
    let parts: Vec<&str> = host.split('.').collect();

    // If we have at least 3 parts (subdomain.domain.tld), return the subdomain
    if parts.len() >= 3 {
        Some(parts[0].to_string())
    } else {
        None
    }
}

/// Sanitize tenant ID to prevent injection attacks
/// Only allows alphanumeric, dash, and underscore
fn sanitize_tenant_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(50) // Limit length
        .collect::<String>()
        .to_lowercase()
}

/// Get store name for a tenant
/// This maps tenant IDs to KV store names
pub fn get_store_name(tenant_id: &str) -> String {
    // Each tenant gets its own store for complete isolation
    // Just return the tenant ID directly (e.g., "sdny", "edny")
    tenant_id.to_lowercase()
}

/// Check if a user has access to a specific tenant
/// This would integrate with your authentication system
///
/// Currently unused but kept for future access control implementation.
/// When authentication is added, this will verify user permissions for specific districts.
#[allow(dead_code)]
pub fn has_tenant_access(_user_id: &str, _tenant_id: &str) -> bool {
    // TODO: Implement actual access control
    // For now, allow all access
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_tenant_id() {
        assert_eq!(sanitize_tenant_id("SDNY"), "sdny");
        assert_eq!(sanitize_tenant_id("district-9"), "district-9");
        assert_eq!(sanitize_tenant_id("court_123"), "court_123");
        assert_eq!(sanitize_tenant_id("bad!@#$%^&*()"), "bad");
        assert_eq!(
            sanitize_tenant_id("very_long_tenant_id_that_exceeds_fifty_characters_limit_here"),
            "very_long_tenant_id_that_exceeds_fifty_characters"
        );
    }

    #[test]
    fn test_extract_subdomain() {
        assert_eq!(extract_subdomain("sdny.lexodus.gov"), Some("sdny".to_string()));
        assert_eq!(extract_subdomain("court.example.com:3000"), Some("court".to_string()));
        assert_eq!(extract_subdomain("localhost"), None);
        assert_eq!(extract_subdomain("example.com"), None);
    }

    #[test]
    fn test_get_store_name() {
        assert_eq!(get_store_name("sdny"), "tenant_sdny");
        assert_eq!(get_store_name("default"), "tenant_default");
    }
}