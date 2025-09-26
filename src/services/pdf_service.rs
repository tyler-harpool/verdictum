use std::sync::Arc;
use crate::domain::document::{
    CourtDocument, GeneratedDocument, DocumentError
};
use crate::ports::document_generator::{DocumentGenerator, DocumentRequest};
use crate::ports::document_repository::DocumentRepository;
use crate::ports::signature_repository::SignatureRepository;
use crate::adapters::pdf_writer_adapter::PdfWriterAdapter;
use crate::adapters::spin_kv_document_repository::SpinKvDocumentRepository;
use crate::adapters::spin_kv_signature_repository::SpinKvSignatureRepository;

/// PDF service that coordinates between ports and adapters following hexagonal architecture
///
/// This service implements the application layer in our hexagonal architecture,
/// orchestrating the generation, storage, and retrieval of court documents and signatures.
///
/// # Architecture
/// - Uses dependency injection to work with abstract ports (traits)
/// - Supports both synchronous and asynchronous operations
/// - Multi-tenant by design (tenant_id determines which KV store to use)
///
/// # Example
/// ```
/// let service = PdfService::new("sdny")?;
/// let doc = service.generate_document_sync(request)?;
/// ```
pub struct PdfService {
    /// Generates PDF documents from court data
    generator: Arc<dyn DocumentGenerator>,
    /// Stores and retrieves generated documents
    repository: Arc<dyn DocumentRepository>,
    /// Manages judge electronic signatures
    signature_repo: Arc<dyn SignatureRepository>,
    /// Tenant ID for multi-tenant data isolation
    tenant_id: String,
}

impl PdfService {
    /// Create a new PDF service with default implementations
    pub fn new(tenant_id: &str) -> Result<Self, DocumentError> {
        // Create concrete implementations
        let generator = Arc::new(PdfWriterAdapter::new()) as Arc<dyn DocumentGenerator>;

        let repository = SpinKvDocumentRepository::new(tenant_id)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to create document repository: {}", e)))?;
        let repository = Arc::new(repository) as Arc<dyn DocumentRepository>;

        let signature_repo = SpinKvSignatureRepository::new(tenant_id)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to create signature repository: {}", e)))?;
        let signature_repo = Arc::new(signature_repo) as Arc<dyn SignatureRepository>;

        Ok(Self {
            generator,
            repository,
            signature_repo,
            tenant_id: tenant_id.to_string(),
        })
    }

    /// Create a service with custom implementations (useful for testing)
    pub fn with_implementations(
        generator: Arc<dyn DocumentGenerator>,
        repository: Arc<dyn DocumentRepository>,
        signature_repo: Arc<dyn SignatureRepository>,
        tenant_id: String,
    ) -> Self {
        Self {
            generator,
            repository,
            signature_repo,
            tenant_id,
        }
    }

    /// Generate a single document (sync wrapper for Spin handlers)
    pub fn generate_document_sync(&self, request: DocumentRequest) -> Result<GeneratedDocument, DocumentError> {
        let document = request.to_court_document();
        // Since PdfWriterAdapter's methods are actually sync, we can call the sync version
        let adapter = PdfWriterAdapter::new();
        adapter.generate_document_sync(document)
    }

    /// Generate multiple documents (sync wrapper for Spin handlers)
    pub fn generate_batch_sync(&self, requests: Vec<DocumentRequest>) -> Result<Vec<GeneratedDocument>, DocumentError> {
        let documents: Vec<CourtDocument> = requests.into_iter()
            .map(|r| r.to_court_document())
            .collect();

        let adapter = PdfWriterAdapter::new();
        adapter.generate_batch_sync(documents)
    }

    /// Generate a single document (async version for future use)
    pub async fn generate_document(&self, request: DocumentRequest) -> Result<GeneratedDocument, DocumentError> {
        let document = request.to_court_document();
        let generated = self.generator.generate_document(document).await?;

        // Optionally persist the document
        // self.repository.save_document(&generated).await?;

        Ok(generated)
    }

    /// Generate multiple documents (async version for future use)
    pub async fn generate_batch(&self, requests: Vec<DocumentRequest>) -> Result<Vec<GeneratedDocument>, DocumentError> {
        let documents: Vec<CourtDocument> = requests.into_iter()
            .map(|r| r.to_court_document())
            .collect();

        let generated = self.generator.generate_batch(documents).await?;

        // Optionally persist all documents
        // for doc in &generated {
        //     self.repository.save_document(doc).await?;
        // }

        Ok(generated)
    }

    /// Store a judge's signature (sync wrapper for Spin handlers)
    pub fn store_signature_sync(&self, judge_id: uuid::Uuid, signature_base64: &str) -> Result<(), DocumentError> {
        // Create a simple sync implementation that stores in KV store
        use spin_sdk::key_value::Store;
        use chrono::Utc;

        // CRITICAL: Use tenant-specific store, not default!
        if self.tenant_id.is_empty() {
            return Err(DocumentError::GenerationFailed("TENANT_NOT_SPECIFIED: tenant ID is required".to_string()));
        }
        let store = Store::open(&self.tenant_id)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to open tenant store '{}': {:?}", self.tenant_id, e)))?;

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(signature_base64.as_bytes());
        let hash = hasher.finalize();

        let signature = crate::ports::signature_repository::JudgeSignature {
            judge_id,
            signature_base64: signature_base64.to_string(),
            uploaded_at: Utc::now().to_rfc3339(),
            signature_hash: format!("{:x}", hash),
        };

        let key = format!("signature_{}", judge_id);
        let value = serde_json::to_vec(&signature)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to serialize: {}", e)))?;

        store.set(&key, &value)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to store: {:?}", e)))?;

        Ok(())
    }

    /// Get a judge's stored signature (sync wrapper for Spin handlers)
    pub fn get_signature_sync(&self, judge_id: uuid::Uuid) -> Result<Option<crate::ports::signature_repository::JudgeSignature>, DocumentError> {
        use spin_sdk::key_value::Store;

        // CRITICAL: Use tenant-specific store, not default!
        if self.tenant_id.is_empty() {
            return Err(DocumentError::GenerationFailed("TENANT_NOT_SPECIFIED: tenant ID is required".to_string()));
        }
        let store = Store::open(&self.tenant_id)
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to open tenant store '{}': {:?}", self.tenant_id, e)))?;

        let key = format!("signature_{}", judge_id);

        match store.get(&key) {
            Ok(Some(data)) => {
                let signature = serde_json::from_slice(&data)
                    .map_err(|e| DocumentError::GenerationFailed(format!("Failed to deserialize: {}", e)))?;
                Ok(Some(signature))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(DocumentError::GenerationFailed(format!("Failed to retrieve: {:?}", e)))
        }
    }

    /// Store a judge's signature (async version for future use)
    pub async fn store_signature(&self, judge_id: uuid::Uuid, signature_base64: &str) -> Result<(), DocumentError> {
        self.signature_repo
            .store_signature(judge_id, signature_base64)
            .await
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to store signature: {:?}", e)))
    }

    /// Get a judge's stored signature (async version for future use)
    pub async fn get_signature(&self, judge_id: uuid::Uuid) -> Result<Option<crate::ports::signature_repository::JudgeSignature>, DocumentError> {
        self.signature_repo
            .get_signature(judge_id)
            .await
            .map_err(|e| DocumentError::GenerationFailed(format!("Failed to get signature: {:?}", e)))
    }
}

/// Factory function to create PDF service based on tenant
pub fn create_pdf_service(tenant_id: &str) -> Result<PdfService, DocumentError> {
    PdfService::new(tenant_id)
}