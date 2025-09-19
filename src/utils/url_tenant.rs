//! URL-based tenant extraction utilities
//!
//! This module provides utilities for extracting tenant information from URL paths
//! instead of headers, supporting RESTful API design patterns.

use spin_sdk::http::Request;

/// Extract tenant ID from URL path
///
/// Supports multiple URL patterns:
/// - `/api/courts/{district}/...` - Simple district-based routing
/// - `/api/courts/{type}/{district}/...` - Hierarchical routing
/// - `/api/{district}/...` - Short form (future)
///
/// # Examples
/// ```
/// assert_eq!(extract_tenant_from_path("/api/courts/sdny/cases"), Some("sdny"));
/// assert_eq!(extract_tenant_from_path("/api/courts/district/sdny/cases"), Some("sdny"));
/// assert_eq!(extract_tenant_from_path("/api/cases"), None);
/// ```
pub fn extract_tenant_from_path(path: &str) -> Option<String> {
    let path = path.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    // Pattern: /api/courts/{district}/...
    if parts.len() >= 3 && parts[0] == "api" && parts[1] == "courts" {
        // Check if third part is a court type (district, bankruptcy, etc.)
        let court_types = ["district", "bankruptcy", "appellate", "fisa", "tax", "claims", "trade"];

        if parts.len() >= 4 && court_types.contains(&parts[2]) {
            // Pattern: /api/courts/{type}/{district}/...
            return Some(sanitize_tenant_id(parts[3]));
        } else {
            // Pattern: /api/courts/{district}/...
            return Some(sanitize_tenant_id(parts[2]));
        }
    }

    None
}

/// Extract court type from URL path
///
/// Returns the court type if specified in the URL, otherwise None
///
/// # Examples
/// ```
/// assert_eq!(extract_court_type_from_path("/api/courts/district/sdny/cases"), Some("district"));
/// assert_eq!(extract_court_type_from_path("/api/courts/bankruptcy/nybk/cases"), Some("bankruptcy"));
/// assert_eq!(extract_court_type_from_path("/api/courts/sdny/cases"), None);
/// ```
pub fn extract_court_type_from_path(path: &str) -> Option<String> {
    let path = path.trim_start_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    let court_types = ["district", "bankruptcy", "appellate", "fisa", "tax", "claims", "trade"];

    if parts.len() >= 4 && parts[0] == "api" && parts[1] == "courts" && court_types.contains(&parts[2]) {
        return Some(parts[2].to_string());
    }

    None
}

/// Extract tenant ID from request
///
/// First tries to extract from URL path, then falls back to headers
/// This allows gradual migration from header-based to URL-based routing
pub fn get_tenant_from_request(req: &Request) -> Result<String, String> {
    // First try URL path
    if let Some(tenant) = extract_tenant_from_path(&req.path()) {
        return Ok(tenant);
    }

    // Fall back to header for backward compatibility
    if let Some(header) = req.header("x-court-district") {
        if let Some(value) = header.as_str() {
            if !value.is_empty() {
                return Ok(sanitize_tenant_id(value));
            }
        }
    }

    Err("Tenant not specified in URL or headers".to_string())
}

/// Sanitize tenant ID to prevent injection attacks
fn sanitize_tenant_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(50)
        .collect::<String>()
        .to_lowercase()
}

/// Build a tenant-scoped URL
///
/// Converts a generic path to a tenant-specific path
///
/// # Examples
/// ```
/// assert_eq!(build_tenant_url("sdny", "/cases"), "/api/courts/sdny/cases");
/// assert_eq!(build_tenant_url("nybk", "/docket/entries"), "/api/courts/nybk/docket/entries");
/// ```
pub fn build_tenant_url(tenant: &str, resource: &str) -> String {
    let resource = resource.trim_start_matches('/');
    format!("/api/courts/{}/{}", tenant, resource)
}

/// Build a hierarchical tenant-scoped URL
///
/// Converts a generic path to a tenant-specific path with court type
///
/// # Examples
/// ```
/// assert_eq!(build_hierarchical_url("district", "sdny", "/cases"), "/api/courts/district/sdny/cases");
/// ```
pub fn build_hierarchical_url(court_type: &str, tenant: &str, resource: &str) -> String {
    let resource = resource.trim_start_matches('/');
    format!("/api/courts/{}/{}/{}", court_type, tenant, resource)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tenant_from_simple_path() {
        assert_eq!(extract_tenant_from_path("/api/courts/sdny/cases"), Some("sdny".to_string()));
        assert_eq!(extract_tenant_from_path("/api/courts/edny/judges"), Some("edny".to_string()));
        assert_eq!(extract_tenant_from_path("/api/courts/NDCA/config"), Some("ndca".to_string()));
    }

    #[test]
    fn test_extract_tenant_from_hierarchical_path() {
        assert_eq!(extract_tenant_from_path("/api/courts/district/sdny/cases"), Some("sdny".to_string()));
        assert_eq!(extract_tenant_from_path("/api/courts/bankruptcy/nybk/cases"), Some("nybk".to_string()));
        assert_eq!(extract_tenant_from_path("/api/courts/appellate/ca2/opinions"), Some("ca2".to_string()));
    }

    #[test]
    fn test_extract_court_type() {
        assert_eq!(extract_court_type_from_path("/api/courts/district/sdny/cases"), Some("district".to_string()));
        assert_eq!(extract_court_type_from_path("/api/courts/bankruptcy/nybk/filings"), Some("bankruptcy".to_string()));
        assert_eq!(extract_court_type_from_path("/api/courts/sdny/cases"), None);
    }

    #[test]
    fn test_invalid_paths() {
        assert_eq!(extract_tenant_from_path("/api/cases"), None);
        assert_eq!(extract_tenant_from_path("/courts/sdny/cases"), None);
        assert_eq!(extract_tenant_from_path(""), None);
        assert_eq!(extract_tenant_from_path("/"), None);
    }

    #[test]
    fn test_build_urls() {
        assert_eq!(build_tenant_url("sdny", "/cases"), "/api/courts/sdny/cases");
        assert_eq!(build_tenant_url("sdny", "cases"), "/api/courts/sdny/cases");
        assert_eq!(build_hierarchical_url("district", "sdny", "/cases"), "/api/courts/district/sdny/cases");
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize_tenant_id("SDNY"), "sdny");
        assert_eq!(sanitize_tenant_id("test@#$%"), "test");
        assert_eq!(sanitize_tenant_id("valid-district_123"), "valid-district_123");
    }
}