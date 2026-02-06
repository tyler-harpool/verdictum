//! Defendant domain model for federal criminal cases
//!
//! Replaces the previous Vec<String> approach with a proper entity
//! containing federal-specific identifiers, custody, bond, and charge information.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Citizenship status for a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CitizenshipStatus {
    UsCitizen,
    LawfulPermanentResident,
    NonImmigrantVisa,
    Undocumented,
    Unknown,
}

/// Custody status for a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CustodyStatus {
    InCustody,
    ReleasedOnBond,
    ReleasedOnOwnRecognizance,
    Fugitive,
    Surrendered,
    HomeConfinement,
    Unknown,
}

/// Type of bail/bond set for a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BailType {
    PersonalRecognizance,
    UnsecuredBond,
    CashBond,
    SuretyBond,
    PropertyBond,
    Detained,
    None,
}

/// Plea type per count (Federal Rules of Criminal Procedure Rule 11)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PleaType {
    Guilty,
    NotGuilty,
    NoloContendere,
    /// No plea has been entered yet
    NotEntered,
}

/// Verdict per count after trial
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Guilty,
    NotGuilty,
    Mistrial,
    Dismissed,
    Pending,
}

/// A single criminal count/charge against a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Count {
    /// Count number as listed in the indictment
    pub count_number: u32,
    /// Federal statute citation (e.g., "18 U.S.C. 1343")
    pub statute: String,
    /// Description of the charged offense
    pub offense_description: String,
    /// Maximum sentence in months under the statute
    pub statutory_max_months: Option<u32>,
    /// Mandatory minimum sentence in months
    pub statutory_min_months: Option<u32>,
    /// Plea entered for this count
    pub plea: PleaType,
    /// When the plea was entered
    pub plea_date: Option<DateTime<Utc>>,
    /// Trial verdict for this count
    pub verdict: Verdict,
    /// When the verdict was entered
    pub verdict_date: Option<DateTime<Utc>>,
}

/// Bond/bail information for a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BondInfo {
    pub bail_type: BailType,
    pub bail_amount: Option<f64>,
    pub conditions_of_release: Vec<String>,
    pub bond_posted_date: Option<DateTime<Utc>>,
    pub surety_name: Option<String>,
}

/// A defendant in a federal criminal case
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Defendant {
    pub id: Uuid,
    pub case_id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    /// U.S. Marshals Service number
    pub usm_number: Option<String>,
    /// FBI identification number
    pub fbi_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub citizenship_status: CitizenshipStatus,
    pub custody_status: CustodyStatus,
    pub bond_info: Option<BondInfo>,
    /// Counts/charges against this defendant
    pub counts: Vec<Count>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to add a defendant to a case
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateDefendantRequest {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub usm_number: Option<String>,
    pub fbi_number: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub citizenship_status: Option<CitizenshipStatus>,
    pub custody_status: Option<CustodyStatus>,
    pub bond_info: Option<BondInfo>,
}

/// Request to add a count/charge to a defendant
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddCountRequest {
    pub count_number: u32,
    pub statute: String,
    pub offense_description: String,
    pub statutory_max_months: Option<u32>,
    pub statutory_min_months: Option<u32>,
}

/// Request to enter a plea for a specific count
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct EnterCountPleaRequest {
    pub defendant_id: Uuid,
    pub count_number: u32,
    pub plea: PleaType,
}

impl Defendant {
    /// Create a new defendant with minimal required fields
    pub fn new(case_id: Uuid, name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            case_id,
            name,
            aliases: Vec::new(),
            usm_number: None,
            fbi_number: None,
            date_of_birth: None,
            citizenship_status: CitizenshipStatus::Unknown,
            custody_status: CustodyStatus::Unknown,
            bond_info: None,
            counts: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Create a defendant from a full request
    pub fn from_request(case_id: Uuid, request: CreateDefendantRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            case_id,
            name: request.name,
            aliases: request.aliases,
            usm_number: request.usm_number,
            fbi_number: request.fbi_number,
            date_of_birth: request.date_of_birth,
            citizenship_status: request.citizenship_status.unwrap_or(CitizenshipStatus::Unknown),
            custody_status: request.custody_status.unwrap_or(CustodyStatus::Unknown),
            bond_info: request.bond_info,
            counts: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Enter a plea for a specific count
    pub fn enter_plea(&mut self, count_number: u32, plea: PleaType) -> Result<(), String> {
        match self.counts.iter_mut().find(|c| c.count_number == count_number) {
            Some(count) => {
                count.plea = plea;
                count.plea_date = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
            None => Err(format!(
                "Count {} not found for defendant {}",
                count_number, self.id
            )),
        }
    }

    /// Add a count/charge to this defendant
    pub fn add_count(&mut self, request: AddCountRequest) {
        let count = Count {
            count_number: request.count_number,
            statute: request.statute,
            offense_description: request.offense_description,
            statutory_max_months: request.statutory_max_months,
            statutory_min_months: request.statutory_min_months,
            plea: PleaType::NotEntered,
            plea_date: None,
            verdict: Verdict::Pending,
            verdict_date: None,
        };
        self.counts.push(count);
        self.updated_at = Utc::now();
    }
}
