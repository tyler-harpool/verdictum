//! Tenant management utilities for multi-tenant support
//!
//! This module handles tenant identification and store selection
//! based on court district or other tenant identifiers.
//!
//! ## Multi-Tenancy Architecture
//!
//! The Federal Court Case Management System uses a multi-tenant architecture where:
//! - Each federal court district operates as an isolated tenant
//! - Data is completely segregated between districts using separate KV stores
//! - Districts can have custom configurations and business rules
//!
//! ## Tenant Identification Methods
//!
//! Tenants are identified through (in order of precedence):
//! 1. `X-Tenant-ID` header - Explicit tenant selection
//! 2. `X-Court-District` header - Standard district identification
//! 3. Host subdomain - e.g., `sdny.lexodus.gov`
//! 4. Query parameter - `?tenant=district9`
//!
//! ## Security Considerations
//!
//! - NEVER defaults to a real tenant to prevent data leakage
//! - Returns "TENANT_NOT_SPECIFIED" when no tenant is found
//! - Invalid tenants map to "UNKNOWN_TENANT_xxx" stores that will fail to open
//! - All tenant IDs are sanitized to prevent injection attacks

use spin_sdk::http::Request;

/// Extract tenant identifier from request
///
/// ## Extraction Order (first match wins):
/// 1. `X-Tenant-ID` header - For explicit tenant selection in testing/admin
/// 2. `X-Court-District` header - Standard header for production use
/// 3. Host header subdomain - e.g., `sdny.lexodus.gov` extracts "sdny"
/// 4. Query parameter - `?tenant=district9` for debugging/testing
///
/// ## Returns
/// - Valid tenant ID if found (e.g., "sdny", "district9")
/// - "TENANT_NOT_SPECIFIED" if no tenant identifier found (SECURITY: prevents default access)
///
/// ## Example Usage
/// ```rust
/// let tenant_id = get_tenant_id(&request);
/// match tenant_id.as_str() {
///     "TENANT_NOT_SPECIFIED" => return error_response("District required"),
///     valid_tenant => // proceed with valid tenant
/// }
/// ```
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

    // CRITICAL SECURITY: No tenant specified - return a special value
    // This MUST NOT default to any real tenant to prevent unauthorized data access
    // The repository layer will reject "TENANT_NOT_SPECIFIED" with AccessDenied
    "TENANT_NOT_SPECIFIED".to_string()
}

/// Extract subdomain from host header
///
/// Parses host header to extract the first subdomain component.
/// Used for domain-based tenant routing (e.g., `sdny.lexodus.gov`).
///
/// ## Examples
/// - `sdny.lexodus.gov` -> `Some("sdny")`
/// - `court.example.com:3000` -> `Some("court")`
/// - `localhost` -> `None`
/// - `example.com` -> `None`
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
///
/// ## Security Features:
/// - Only allows alphanumeric characters, dashes, and underscores
/// - Converts to lowercase for consistency
/// - Limits to 50 characters to prevent overflow
/// - Removes any potentially dangerous characters
///
/// ## Examples
/// - `"SDNY"` -> `"sdny"`
/// - `"district-9"` -> `"district-9"`
/// - `"bad!@#$%"` -> `"bad"`
/// - Very long strings are truncated to 50 chars
fn sanitize_tenant_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(50) // Limit length
        .collect::<String>()
        .to_lowercase()
}

/// List of valid federal court districts
///
/// ## Production Districts:
/// - `sdny` - Southern District of New York
/// - `edny` - Eastern District of New York
/// - `ndca` - Northern District of California
/// - `cdca` - Central District of California
/// - `sdtx` - Southern District of Texas
/// - `ndil` - Northern District of Illinois
/// - `ddc` - District of D.C.
/// - `fisa` - FISA Court
/// - `tax` - Tax Court
/// - And more...
const FEDERAL_COURTS: &[&str] = &[
    "sdny", "edny", "ndca", "cdca", "sdtx", "ndil", "ddc",
    "ndny", "wdny", "nybk", "edtx", "fisa", "tax", "generic",
];

/// List of test districts (movie references)
///
/// ## Test Districts:
/// - `district9` - From "District 9" movie (alien refugee camp)
/// - `district12` - From "Hunger Games" (coal mining district)
///
/// These are used for testing and development to avoid
/// confusion with real federal court districts.
const TEST_DISTRICTS: &[&str] = &[
    "district9",  // From District 9 movie
    "district12", // From Hunger Games
];

/// Check if a tenant ID is a valid district
///
/// Validates against both production federal courts and test districts.
/// Also allows "default" and "test" for backward compatibility.
///
/// ## Returns
/// - `true` if the tenant ID matches a known district
/// - `false` for unknown or invalid districts
fn is_valid_district(tenant_id: &str) -> bool {
    let lower = tenant_id.to_lowercase();
    FEDERAL_COURTS.contains(&lower.as_str()) ||
    TEST_DISTRICTS.contains(&lower.as_str()) ||
    lower == "default" ||
    lower == "test"
}

/// Get store name for a tenant
///
/// Maps tenant IDs to KV store names for data isolation.
/// Each district gets its own completely isolated store.
///
/// ## Store Naming Convention:
/// - Valid districts -> lowercase district name (e.g., "sdny")
/// - Empty/not specified -> "tenant_not_specified" (fails with AccessDenied)
/// - Unknown tenants -> "UNKNOWN_TENANT_xxx" (fails to open)
///
/// ## Security:
/// The special store names "tenant_not_specified" and "UNKNOWN_TENANT_xxx"
/// are designed to fail when `Store::open()` is called, preventing any
/// data access when tenant validation fails.
///
/// ## Examples:
/// ```
/// get_store_name("SDNY") -> "sdny"
/// get_store_name("district9") -> "district9"
/// get_store_name("") -> "tenant_not_specified"
/// get_store_name("invalid") -> "UNKNOWN_TENANT_invalid"
/// ```
pub fn get_store_name(tenant_id: &str) -> String {
    // Map tenant IDs to their specific stores
    // Each district gets its own isolated store
    if tenant_id.is_empty() || tenant_id == "TENANT_NOT_SPECIFIED" {
        // SECURITY: Empty or not specified tenant must fail safely
        // This store name will cause Store::open() to fail with AccessDenied
        // preventing any unauthorized data access
        return "tenant_not_specified".to_string();
    }

    let tenant_lower = tenant_id.to_lowercase();

    // For valid districts, use the district name as the store name
    // This allows easy addition of new districts
    if is_valid_district(&tenant_lower) {
        tenant_lower
    } else {
        // SECURITY: Unknown tenants get a store name that will fail to open
        // This prevents typos or invalid districts from accessing any real data
        format!("UNKNOWN_TENANT_{}", tenant_lower)
    }
}

/// Validate that a tenant ID is specified and valid
///
/// Performs comprehensive validation of tenant ID from request.
/// This should be called early in request processing to fail fast.
///
/// ## Validation Steps:
/// 1. Extract tenant ID from request
/// 2. Check if tenant was found (not empty or "TENANT_NOT_SPECIFIED")
/// 3. Verify tenant maps to a valid store name
/// 4. Ensure tenant is a known district
///
/// ## Returns
/// - `Ok(tenant_id)` - Valid tenant ID ready for use
/// - `Err(message)` - Validation failure with user-friendly error message
///
/// ## Example Usage:
/// ```rust
/// match validate_tenant_id(&req) {
///     Ok(tenant) => proceed_with_tenant(tenant),
///     Err(msg) => return error_response(&msg),
/// }
/// ```
pub fn validate_tenant_id(req: &Request) -> Result<String, String> {
    let tenant_id = get_tenant_id(req);

    // Check if tenant was actually found
    if tenant_id.is_empty() || tenant_id == "" {
        return Err("Missing required header: X-Court-District or X-Tenant-ID".to_string());
    }

    // Check if it's a valid district
    let store_name = get_store_name(&tenant_id);
    if store_name.starts_with("TENANT_NOT_SPECIFIED") {
        return Err("Missing required header: X-Court-District or X-Tenant-ID".to_string());
    }

    if store_name.starts_with("UNKNOWN_TENANT_") {
        return Err(format!("Invalid district: {}", tenant_id));
    }

    Ok(tenant_id)
}

/// Check if a user has access to a specific tenant
///
/// ## Future Implementation:
/// This function is a placeholder for future role-based access control (RBAC).
/// When authentication is added, this will:
/// - Verify user has permission to access the specified district
/// - Check user roles (e.g., judge, clerk, attorney)
/// - Validate cross-district access for special roles
///
/// ## Current Behavior:
/// Returns `true` for all requests (no access control)
///
/// ## Future Example:
/// ```rust
/// // Judge can access their home district + visiting districts
/// if user.role == "judge" {
///     return user.districts.contains(tenant_id);
/// }
/// // Attorneys may have limited cross-district access
/// if user.role == "attorney" {
///     return user.has_case_in_district(tenant_id);
/// }
/// ```
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
        // Test production districts map to their own stores
        assert_eq!(get_store_name("sdny"), "sdny");
        assert_eq!(get_store_name("edny"), "edny");

        // Test movie districts map to their own stores
        assert_eq!(get_store_name("district9"), "district9");
        assert_eq!(get_store_name("District12"), "district12");  // Case insensitive

        // Test error conditions
        assert_eq!(get_store_name(""), "TENANT_NOT_SPECIFIED");
        assert_eq!(get_store_name("invalid"), "UNKNOWN_TENANT_invalid");
    }
}