use async_trait::async_trait;
use crate::domain::document::{
    CourtDocument, GeneratedDocument, DocumentError, DocumentMetadata,
    CaseNumber, JudgeName, District, DocumentType, ElectronicSignature
};

#[async_trait]
pub trait DocumentGenerator: Send + Sync {
    async fn generate_document(&self, document: CourtDocument) -> Result<GeneratedDocument, DocumentError>;

    async fn generate_batch(&self, documents: Vec<CourtDocument>) -> Result<Vec<GeneratedDocument>, DocumentError>;
}

#[async_trait]
pub trait PdfRenderer: Send + Sync {
    fn render_rule16b(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        signature: Option<&ElectronicSignature>
    ) -> Result<Vec<u8>, DocumentError>;

    fn render_court_order(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        order_title: &str,
        order_content: &str,
        signature: Option<&ElectronicSignature>
    ) -> Result<Vec<u8>, DocumentError>;

    fn render_minute_entry(
        &self,
        case_number: &CaseNumber,
        defendant_names: &str,
        judge_name: &JudgeName,
        district: &District,
        minute_text: &str
    ) -> Result<Vec<u8>, DocumentError>;

    fn render_waiver_indictment(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        charges: &str
    ) -> Result<Vec<u8>, DocumentError>;

    fn render_conditions_release(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        judge_name: &JudgeName,
        conditions: &[String]
    ) -> Result<Vec<u8>, DocumentError>;

    fn render_criminal_judgment(
        &self,
        case_number: &CaseNumber,
        defendant_name: &str,
        district: &District,
        judge_name: &JudgeName,
        plea: &str,
        counts: &str,
        sentence: &str
    ) -> Result<Vec<u8>, DocumentError>;
}

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn save_document(&self, document: &GeneratedDocument) -> Result<(), DocumentError>;

    async fn get_document_by_id(&self, document_id: &str) -> Result<Option<GeneratedDocument>, DocumentError>;

    async fn list_documents_by_case(&self, case_number: &CaseNumber) -> Result<Vec<CourtDocument>, DocumentError>;
}

pub struct DocumentRequest {
    pub case_number: CaseNumber,
    pub document_type: DocumentType,
    pub district: District,
    pub metadata: DocumentMetadata,
}

impl DocumentRequest {
    pub fn to_court_document(self) -> CourtDocument {
        CourtDocument {
            id: crate::domain::document::DocumentId::new(),
            case_number: self.case_number,
            document_type: self.document_type,
            district: self.district,
            created_at: chrono::Utc::now(),
            metadata: self.metadata,
        }
    }
}