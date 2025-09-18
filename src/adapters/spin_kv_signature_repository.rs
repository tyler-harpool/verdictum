//! Spin KV adapter for judge signature storage

use async_trait::async_trait;
use spin_sdk::key_value::{Store, Error as KvError};
use uuid::Uuid;
use crate::ports::signature_repository::{SignatureRepository, JudgeSignature, SignatureError};

pub struct SpinKvSignatureRepository {
    store: Store,
}

impl SpinKvSignatureRepository {
    pub fn new(tenant_id: &str) -> Result<Self, KvError> {
        let store_name = format!("{}_signatures", tenant_id);
        let store = Store::open(&store_name)?;
        Ok(Self { store })
    }
}

#[async_trait]
impl SignatureRepository for SpinKvSignatureRepository {
    async fn store_signature(&self, judge_id: Uuid, signature_base64: &str) -> Result<(), SignatureError> {
        use chrono::Utc;
        use sha2::{Sha256, Digest};

        // Calculate hash for verification
        let mut hasher = Sha256::new();
        hasher.update(signature_base64.as_bytes());
        let signature_hash = format!("{:x}", hasher.finalize());

        let signature = JudgeSignature {
            judge_id,
            signature_base64: signature_base64.to_string(),
            uploaded_at: Utc::now().to_rfc3339(),
            signature_hash,
        };

        let key = format!("judge_sig_{}", judge_id);
        let value = serde_json::to_vec(&signature)
            .map_err(|e| SignatureError::SerializationError(e.to_string()))?;

        self.store.set(&key, &value)
            .map_err(|e| SignatureError::StorageError(e.to_string()))?;
        Ok(())
    }

    async fn get_signature(&self, judge_id: Uuid) -> Result<Option<JudgeSignature>, SignatureError> {
        let key = format!("judge_sig_{}", judge_id);

        match self.store.get(&key).map_err(|e| SignatureError::StorageError(e.to_string()))? {
            Some(data) => {
                let signature = serde_json::from_slice(&data)
                    .map_err(|e| SignatureError::SerializationError(e.to_string()))?;
                Ok(Some(signature))
            }
            None => Ok(None)
        }
    }

    async fn delete_signature(&self, judge_id: Uuid) -> Result<(), SignatureError> {
        let key = format!("judge_sig_{}", judge_id);
        self.store.delete(&key)
            .map_err(|e| SignatureError::StorageError(e.to_string()))?;
        Ok(())
    }

    async fn verify_signature(&self, judge_id: Uuid, signature_base64: &str) -> Result<bool, SignatureError> {
        use sha2::{Sha256, Digest};

        if let Some(stored_sig) = self.get_signature(judge_id).await? {
            let mut hasher = Sha256::new();
            hasher.update(signature_base64.as_bytes());
            let calculated_hash = format!("{:x}", hasher.finalize());

            Ok(calculated_hash == stored_sig.signature_hash)
        } else {
            Ok(false)
        }
    }

    async fn list_signatures(&self) -> Result<Vec<JudgeSignature>, SignatureError> {
        let keys = self.store.get_keys()
            .map_err(|e| SignatureError::StorageError(e.to_string()))?;

        let mut signatures = Vec::new();
        for key in keys {
            if key.starts_with("judge_sig_") {
                if let Some(data) = self.store.get(&key)
                    .map_err(|e| SignatureError::StorageError(e.to_string()))? {
                    let sig = serde_json::from_slice(&data)
                        .map_err(|e| SignatureError::SerializationError(e.to_string()))?;
                    signatures.push(sig);
                }
            }
        }
        Ok(signatures)
    }
}