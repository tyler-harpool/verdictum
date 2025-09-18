//! Port for judge signature storage and retrieval

use async_trait::async_trait;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeSignature {
    pub judge_id: Uuid,
    pub signature_base64: String,
    pub uploaded_at: String,
    pub signature_hash: String, // SHA256 hash for verification
}

#[derive(Debug)]
pub enum SignatureError {
    StorageError(String),
    NotFound,
    InvalidSignature,
    SerializationError(String),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureError::StorageError(e) => write!(f, "Storage error: {}", e),
            SignatureError::NotFound => write!(f, "Signature not found"),
            SignatureError::InvalidSignature => write!(f, "Invalid signature"),
            SignatureError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for SignatureError {}

/// Port for signature storage operations
#[async_trait]
pub trait SignatureRepository: Send + Sync {
    /// Store a judge's signature
    async fn store_signature(&self, judge_id: Uuid, signature_base64: &str) -> Result<(), SignatureError>;

    /// Retrieve a judge's signature
    async fn get_signature(&self, judge_id: Uuid) -> Result<Option<JudgeSignature>, SignatureError>;

    /// Delete a judge's signature
    async fn delete_signature(&self, judge_id: Uuid) -> Result<(), SignatureError>;

    /// Verify signature integrity
    async fn verify_signature(&self, judge_id: Uuid, signature_base64: &str) -> Result<bool, SignatureError>;

    /// List all stored signatures for a tenant
    async fn list_signatures(&self) -> Result<Vec<JudgeSignature>, SignatureError>;
}