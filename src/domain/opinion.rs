//! Domain models for judicial opinions
//!
//! This module defines the core types for managing court opinions,
//! including published and unpublished decisions, dissents, and concurrences.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a judicial opinion in the system
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JudicialOpinion {
    pub id: String,
    pub case_id: String,
    pub case_name: String,
    pub docket_number: String,
    pub author_judge_id: String,
    pub author_judge_name: String,
    pub opinion_type: OpinionType,
    pub disposition: Disposition,
    pub title: String,
    pub syllabus: String,
    pub content: String,
    pub status: OpinionStatus,
    pub is_published: bool,
    pub is_precedential: bool,
    pub citation: Option<Citation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub filed_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub joining_judges: Vec<JudgeVote>,
    pub related_opinions: Vec<RelatedOpinion>,
    pub legal_citations: Vec<LegalCitation>,
    pub headnotes: Vec<Headnote>,
    pub keywords: Vec<String>,
    pub attachments: Vec<String>,
}

/// Type of judicial opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OpinionType {
    Majority,
    Plurality,
    Concurring,
    ConcurringInPart,
    Dissenting,
    DissentingInPart,
    PerCuriam,
    Memorandum,
    BenchOpinion,
    SlipOpinion,
    PublishedOpinion,
}

/// Disposition of the case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Disposition {
    Affirmed,
    Reversed,
    Remanded,
    AffirmedInPart,
    ReversedInPart,
    Vacated,
    Modified,
    Dismissed,
    Other(String),
}

/// Status of an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OpinionStatus {
    Draft,
    InReview,
    Circulating,
    Final,
    Filed,
    Published,
    Withdrawn,
    Superseded,
}

/// Citation information for an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Citation {
    pub federal_reporter: Option<String>,
    pub federal_supplement: Option<String>,
    pub federal_appendix: Option<String>,
    pub lexis: Option<String>,
    pub westlaw: Option<String>,
    pub neutral_citation: Option<String>,
    pub parallel_citations: Vec<String>,
}

/// Judge vote on an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JudgeVote {
    pub judge_id: String,
    pub judge_name: String,
    pub vote: VoteType,
    pub opinion_id: Option<String>, // If judge wrote separate opinion
    pub notes: String,
}

/// Type of vote on an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VoteType {
    Joins,
    JoinsInPart,
    Concurs,
    ConcursInResult,
    Dissents,
    TakesNoPart,
    NotParticipating,
}

/// Related opinion reference
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RelatedOpinion {
    pub opinion_id: String,
    pub case_name: String,
    pub relationship: OpinionRelationship,
    pub judge_name: String,
}

/// Relationship between opinions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OpinionRelationship {
    Concurring,
    Dissenting,
    Previous,
    Subsequent,
    Related,
}

/// Legal citation within the opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LegalCitation {
    pub citation_text: String,
    pub case_name: String,
    pub reporter: String,
    pub year: Option<i32>,
    pub court: Option<String>,
    pub page: Option<String>,
    pub pin_cite: Option<String>,
    pub proposition: String,
    pub treatment: CitationTreatment,
}

/// Treatment of a cited case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum CitationTreatment {
    Followed,
    Distinguished,
    Explained,
    Harmonized,
    Criticized,
    Questioned,
    Overruled,
    Superseded,
    Cited,
}

/// Headnote for an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Headnote {
    pub number: i32,
    pub topic: String,
    pub subtopic: Option<String>,
    pub text: String,
    pub key_number: Option<String>,
    pub cited_paragraphs: Vec<String>,
}

/// Draft management for opinions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpinionDraft {
    pub id: String,
    pub opinion_id: String,
    pub version: i32,
    pub content: String,
    pub changes_summary: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub comments: Vec<DraftComment>,
    pub is_current: bool,
}

/// Comment on an opinion draft
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DraftComment {
    pub id: String,
    pub judge_id: String,
    pub judge_name: String,
    pub paragraph_ref: Option<String>,
    pub comment_text: String,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Statistics for an opinion
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OpinionStatistics {
    pub word_count: usize,
    pub citation_count: usize,
    pub footnote_count: usize,
    pub times_cited: usize,
    pub download_count: usize,
    pub last_cited: Option<DateTime<Utc>>,
}

impl JudicialOpinion {
    /// Create a new draft opinion
    pub fn new(
        case_id: String,
        case_name: String,
        docket_number: String,
        author_judge_id: String,
        author_judge_name: String,
        opinion_type: OpinionType,
        title: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            case_id,
            case_name,
            docket_number,
            author_judge_id,
            author_judge_name,
            opinion_type,
            disposition: Disposition::Other("Pending".to_string()),
            title,
            syllabus: String::new(),
            content: String::new(),
            status: OpinionStatus::Draft,
            is_published: false,
            is_precedential: false,
            citation: None,
            created_at: now,
            updated_at: now,
            filed_at: None,
            published_at: None,
            joining_judges: Vec::new(),
            related_opinions: Vec::new(),
            legal_citations: Vec::new(),
            headnotes: Vec::new(),
            keywords: Vec::new(),
            attachments: Vec::new(),
        }
    }

    /// File the opinion
    pub fn file(&mut self) {
        self.status = OpinionStatus::Filed;
        self.filed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Publish the opinion
    pub fn publish(&mut self, citation: Citation) {
        self.is_published = true;
        self.status = OpinionStatus::Published;
        self.citation = Some(citation);
        self.published_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Add a joining judge
    pub fn add_joining_judge(&mut self, vote: JudgeVote) {
        self.joining_judges.push(vote);
        self.updated_at = Utc::now();
    }

    /// Add a legal citation
    pub fn add_citation(&mut self, citation: LegalCitation) {
        self.legal_citations.push(citation);
        self.updated_at = Utc::now();
    }

    /// Add a headnote
    pub fn add_headnote(&mut self, headnote: Headnote) {
        self.headnotes.push(headnote);
        self.updated_at = Utc::now();
    }

    /// Check if opinion is a majority opinion
    pub fn is_majority(&self) -> bool {
        matches!(self.opinion_type, OpinionType::Majority | OpinionType::PerCuriam)
    }

    /// Check if opinion creates binding precedent
    pub fn is_binding(&self) -> bool {
        self.is_published && self.is_precedential && self.is_majority()
    }

    /// Calculate opinion statistics
    pub fn calculate_statistics(&self) -> OpinionStatistics {
        OpinionStatistics {
            word_count: self.content.split_whitespace().count(),
            citation_count: self.legal_citations.len(),
            footnote_count: self.content.matches("[^").count(),
            times_cited: 0, // Would be tracked externally
            download_count: 0, // Would be tracked externally
            last_cited: None,
        }
    }
}

impl OpinionDraft {
    /// Create a new draft version
    pub fn new(
        opinion_id: String,
        version: i32,
        content: String,
        changes_summary: String,
        created_by: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            opinion_id,
            version,
            content,
            changes_summary,
            created_by,
            created_at: Utc::now(),
            comments: Vec::new(),
            is_current: true,
        }
    }

    /// Add a comment to the draft
    pub fn add_comment(&mut self, comment: DraftComment) {
        self.comments.push(comment);
    }

    /// Resolve a comment
    pub fn resolve_comment(&mut self, comment_id: &str) {
        if let Some(comment) = self.comments.iter_mut().find(|c| c.id == comment_id) {
            comment.resolved = true;
            comment.resolved_at = Some(Utc::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opinion_creation() {
        let opinion = JudicialOpinion::new(
            "CASE-123".to_string(),
            "Smith v. Jones".to_string(),
            "21-cv-1234".to_string(),
            "JUDGE-456".to_string(),
            "Hon. Jane Doe".to_string(),
            OpinionType::Majority,
            "Opinion of the Court".to_string(),
        );

        assert_eq!(opinion.case_id, "CASE-123");
        assert_eq!(opinion.author_judge_name, "Hon. Jane Doe");
        assert!(matches!(opinion.status, OpinionStatus::Draft));
        assert!(!opinion.is_published);
    }

    #[test]
    fn test_opinion_binding_status() {
        let mut opinion = JudicialOpinion::new(
            "CASE-123".to_string(),
            "Smith v. Jones".to_string(),
            "21-cv-1234".to_string(),
            "JUDGE-456".to_string(),
            "Hon. Jane Doe".to_string(),
            OpinionType::Majority,
            "Opinion of the Court".to_string(),
        );

        assert!(!opinion.is_binding());

        opinion.is_published = true;
        opinion.is_precedential = true;
        assert!(opinion.is_binding());

        opinion.opinion_type = OpinionType::Dissenting;
        assert!(!opinion.is_binding());
    }
}