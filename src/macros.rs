//! Common macros used throughout the application

/// Helper macro to get tenant-specific repository for any repository type
///
/// Usage:
/// ```
/// let repo = get_tenant_repo!(SpinKvDocumentRepository, &req);
/// let repo = get_tenant_repo!(SpinKvJudgeRepository, &req);
/// ```
#[macro_export]
macro_rules! get_tenant_repo {
    ($repo_type:ty, $req:expr) => {{
        let tenant_id = $crate::utils::tenant::get_tenant_id($req);
        let store_name = $crate::utils::tenant::get_store_name(&tenant_id);
        <$repo_type>::with_store(store_name)
    }};
}

/// Helper macro to check tenant access
///
/// Returns an error response if the user doesn't have access to the tenant
#[macro_export]
macro_rules! check_tenant_access {
    ($req:expr, $user_id:expr) => {{
        let tenant_id = $crate::utils::tenant::get_tenant_id($req);
        if !$crate::utils::tenant::has_tenant_access($user_id, &tenant_id) {
            return Err($crate::error::ApiError::Forbidden(
                format!("Access denied to tenant: {}", tenant_id)
            ));
        }
        tenant_id
    }};
}

/// Helper macro for sealed document access control
///
/// Checks if a document is sealed and if the user has permission to access it
#[macro_export]
macro_rules! check_sealed_access {
    ($document:expr, $user_role:expr) => {{
        if $document.is_sealed {
            match $user_role {
                UserRole::Judge | UserRole::Clerk | UserRole::USAttorney => {
                    // Authorized roles can access sealed documents
                },
                _ => {
                    return Err($crate::error::ApiError::Forbidden(
                        "Access denied to sealed document".to_string()
                    ));
                }
            }
        }
    }};
}