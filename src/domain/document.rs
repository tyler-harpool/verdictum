use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentId(Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNumber(String);

impl CaseNumber {
    pub fn new(number: String) -> Result<Self, DocumentError> {
        if number.is_empty() {
            return Err(DocumentError::InvalidCaseNumber);
        }
        Ok(Self(number))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeName(String);

impl JudgeName {
    pub fn new(name: String) -> Result<Self, DocumentError> {
        if name.is_empty() {
            return Err(DocumentError::InvalidJudgeName);
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct District(String);

impl District {
    pub fn new(district: String) -> Result<Self, DocumentError> {
        if district.is_empty() {
            return Err(DocumentError::InvalidDistrict);
        }
        Ok(Self(district))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Rule16b,
    CourtOrder,
    MinuteEntry,
    WaiverIndictment,
    ConditionsRelease,
    CriminalJudgment,
}

#[derive(Debug, Clone)]
pub struct CourtDocument {
    pub id: DocumentId,
    pub case_number: CaseNumber,
    pub document_type: DocumentType,
    pub district: District,
    pub created_at: DateTime<Utc>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone)]
pub enum DocumentMetadata {
    Rule16b {
        defendant_names: String,
        judge_name: JudgeName,
        signature: Option<ElectronicSignature>,
    },
    CourtOrder {
        defendant_names: String,
        judge_name: JudgeName,
        order_title: String,
        order_content: String,
        signature: Option<ElectronicSignature>,
    },
    MinuteEntry {
        defendant_names: String,
        judge_name: JudgeName,
        minute_text: String,
    },
    WaiverIndictment {
        defendant_name: String,
        charges: String,
    },
    ConditionsRelease {
        defendant_name: String,
        judge_name: JudgeName,
        conditions: Vec<String>,
    },
    CriminalJudgment {
        defendant_name: String,
        judge_name: JudgeName,
        plea: String,
        counts: String,
        sentence: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicSignature {
    pub signer_name: String,
    pub signed_at: DateTime<Utc>,
    pub signature_hash: String,
    pub verification_code: String,
}

impl ElectronicSignature {
    pub fn new(signer_name: String, signature_data: &str) -> Self {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(signature_data.as_bytes());
        hasher.update(signer_name.as_bytes());
        let timestamp = Utc::now();
        hasher.update(timestamp.to_string().as_bytes());

        let hash = format!("{:x}", hasher.finalize());
        let verification_code = format!("DOC-{}", &hash[..8].to_uppercase());

        Self {
            signer_name,
            signed_at: timestamp,
            signature_hash: hash,
            verification_code,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedDocument {
    pub document: CourtDocument,
    pub pdf_data: Vec<u8>,
    pub filename: String,
}

impl GeneratedDocument {
    pub fn to_base64(&self) -> String {
        use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
        BASE64.encode(&self.pdf_data)
    }
}

#[derive(Debug, Clone)]
pub enum DocumentError {
    InvalidCaseNumber,
    InvalidJudgeName,
    InvalidDistrict,
    GenerationFailed(String),
}

impl std::fmt::Display for DocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCaseNumber => write!(f, "Invalid case number"),
            Self::InvalidJudgeName => write!(f, "Invalid judge name"),
            Self::InvalidDistrict => write!(f, "Invalid district"),
            Self::GenerationFailed(msg) => write!(f, "Document generation failed: {}", msg),
        }
    }
}

impl std::error::Error for DocumentError {}