//! Federal Sentencing Guidelines and Sentencing Management
//!
//! This module handles federal sentencing calculations, guidelines,
//! departures, variances, and supervised release conditions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Federal sentencing information for a defendant
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Sentencing {
    pub id: String,
    pub case_id: String,
    pub defendant_id: String,
    pub judge_id: String,

    // Guidelines Calculation
    pub offense_level: OffenseLevel,
    pub criminal_history: CriminalHistory,
    pub guidelines_range: GuidelinesRange,

    // Adjustments and Departures
    pub adjustments: Vec<OffenseLevelAdjustment>,
    pub departures: Vec<Departure>,
    pub variance: Option<Variance>,

    // 3553(a) Factors
    pub statutory_factors: StatutoryFactors,

    // Final Sentence
    pub imposed_sentence: Option<ImposedSentence>,
    pub supervised_release: Option<SupervisedRelease>,
    pub special_conditions: Vec<SpecialCondition>,

    // BOP Recommendations
    pub bop_designation: Option<BOPDesignation>,
    pub rdap_eligibility: bool,

    // Administrative
    pub presentence_report_date: Option<DateTime<Utc>>,
    pub sentencing_date: Option<DateTime<Utc>>,
    pub judgment_date: Option<DateTime<Utc>>,
    pub appeal_waiver: bool,
    pub substantial_assistance: Option<SubstantialAssistance>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Offense level calculation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OffenseLevel {
    pub base_offense_level: i32,
    pub specific_offense_characteristics: Vec<OffenseCharacteristic>,
    pub cross_references: Vec<CrossReference>,
    pub adjusted_offense_level: i32,
    pub acceptance_of_responsibility: i32,
    pub obstruction_enhancement: i32,
    pub role_adjustment: i32,
    pub multiple_count_adjustment: Option<MultipleCount>,
    pub final_offense_level: i32,
}

/// Specific offense characteristics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OffenseCharacteristic {
    pub guideline_section: String,
    pub description: String,
    pub adjustment: i32,
    pub rationale: String,
}

/// Cross-reference to other guidelines
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CrossReference {
    pub from_section: String,
    pub to_section: String,
    pub reason: String,
    pub applied: bool,
}

/// Multiple count adjustments
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MultipleCount {
    pub grouping_rules: Vec<String>,
    pub units: i32,
    pub adjustment: i32,
}

/// Criminal history calculation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CriminalHistory {
    pub category: CriminalHistoryCategory,
    pub points: i32,
    pub prior_sentences: Vec<PriorSentence>,
    pub status_points: i32, // Under supervision, on probation, etc.
    pub recency_points: i32,
    pub career_offender: bool,
    pub armed_career_criminal: bool,
}

/// Criminal history categories (I-VI)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CriminalHistoryCategory {
    I,
    II,
    III,
    IV,
    V,
    VI,
}

/// Prior sentence for criminal history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriorSentence {
    pub offense: String,
    pub sentence_date: DateTime<Utc>,
    pub sentence_length_months: i32,
    pub points_assigned: i32,
    pub violence_involved: bool,
    pub controlled_substance: bool,
}

/// Guidelines sentencing range
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GuidelinesRange {
    pub minimum_months: i32,
    pub maximum_months: i32,
    pub zone: Zone,
    pub mandatory_minimum: Option<i32>,
    pub statutory_maximum: Option<i32>,
}

/// Sentencing zones (A, B, C, D)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum Zone {
    A, // Probation eligible
    B, // Split sentence eligible
    C, // Split sentence or prison
    D, // Prison only
}

/// Offense level adjustments
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OffenseLevelAdjustment {
    pub adjustment_type: AdjustmentType,
    pub guideline_section: String,
    pub adjustment_amount: i32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum AdjustmentType {
    VulnerableVictim,
    RoleInOffense,
    ObstructionOfJustice,
    AcceptanceOfResponsibility,
    SubstantialAssistance,
    SafetyValve,
    Other,
}

/// Sentencing enhancements
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Enhancement {
    pub enhancement_type: String,
    pub statute: String,
    pub levels_added: i32,
    pub mandatory_minimum: Option<i32>,
    pub description: String,
}

/// Sentencing reductions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Reduction {
    pub reduction_type: String,
    pub statute: Option<String>,
    pub levels_reduced: i32,
    pub description: String,
}

/// Departures from guidelines
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Departure {
    pub departure_type: DepartureType,
    pub direction: DepartureDirection,
    pub levels: i32,
    pub guideline_section: String,
    pub reason: String,
    pub government_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DepartureType {
    SubstantialAssistance,  // 5K1.1
    FastTrack,              // 5K3.1
    EarlyDisposition,
    AberrantBehavior,
    DiminishedCapacity,
    ExtraordinaryRestitution,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DepartureDirection {
    Upward,
    Downward,
}

/// Variances from guidelines
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Variance {
    pub variance_type: VarianceType,
    pub direction: VarianceDirection,
    pub from_months: i32,
    pub to_months: i32,
    pub percent_change: f32,
    pub factors_considered: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VarianceType {
    Section3553a,
    PolicyDisagreement,
    CriminalHistory,
    Cooperation,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum VarianceDirection {
    Above,
    Below,
    Within,
}

/// 18 U.S.C. § 3553(a) factors
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatutoryFactors {
    pub nature_and_circumstances: String,
    pub defendant_history: String,
    pub seriousness_of_offense: String,
    pub respect_for_law: String,
    pub just_punishment: String,
    pub deterrence: String,
    pub protect_public: String,
    pub defendant_needs: String,
    pub victim_impact: String,
    pub avoid_disparities: String,
    pub restitution_needed: bool,
}

/// Imposed sentence details
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImposedSentence {
    pub custody_months: i32,
    pub probation_months: i32,
    pub home_confinement_months: i32,
    pub intermittent_confinement: bool,
    pub fine_amount: Option<f64>,
    pub restitution_amount: Option<f64>,
    pub forfeiture_amount: Option<f64>,
    pub special_assessment: f64,
    pub concurrent_consecutive: ConcurrentConsecutive,
    pub credit_time_served: i32,
    pub voluntary_surrender_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConcurrentConsecutive {
    Concurrent,
    Consecutive,
    PartiallyConsecutive,
}

/// Supervised release terms
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SupervisedRelease {
    pub term_months: i32,
    pub standard_conditions: bool,
    pub special_conditions: Vec<SpecialCondition>,
    pub drug_testing_required: bool,
    pub computer_monitoring: bool,
    pub financial_disclosure: bool,
    pub employment_requirements: bool,
    pub residence_restrictions: Vec<String>,
    pub association_restrictions: Vec<String>,
    pub travel_restrictions: TravelRestrictions,
}

/// Special conditions of supervision
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpecialCondition {
    pub condition_type: ConditionType,
    pub description: String,
    pub duration_months: Option<i32>,
    pub monitoring_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ConditionType {
    SubstanceAbuseTreatment,
    MentalHealthTreatment,
    VocationalTraining,
    CommunityService,
    HomeDetention,
    ElectronicMonitoring,
    SexOffenderTreatment,
    SexOffenderRegistration,
    ComputerRestrictions,
    FinancialDisclosure,
    GamblingProhibition,
    WeaponsProhibition,
    Other,
}

/// Travel restrictions
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TravelRestrictions {
    pub restricted_to_district: bool,
    pub passport_surrendered: bool,
    pub international_travel_banned: bool,
    pub permission_required: bool,
    pub excluded_areas: Vec<String>,
}

/// Bureau of Prisons designation recommendation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BOPDesignation {
    pub security_level_recommendation: SecurityLevel,
    pub facility_recommendations: Vec<String>,
    pub medical_needs: Vec<String>,
    pub program_needs: Vec<String>,
    pub closer_to_home_request: bool,
    pub rdap_recommendation: bool,
    pub voluntary_surrender_recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum SecurityLevel {
    Minimum,
    Low,
    Medium,
    High,
    Administrative,
}

/// Facility level (alias for SecurityLevel for API compatibility)
pub type FacilityLevel = SecurityLevel;

/// Restitution order
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Restitution {
    pub total_amount: f64,
    pub victims: Vec<String>,
    pub payment_schedule: String,
    pub joint_and_several: bool,
    pub priority_order: i32,
}

/// Consecutive sentence information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConsecutiveSentence {
    pub case_number: String,
    pub months: i32,
    pub reason: String,
}

/// Prior conviction record for criminal history
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PriorConviction {
    pub id: String,
    pub conviction_date: DateTime<Utc>,
    pub offense_description: String,
    pub jurisdiction: String,
    pub sentence_imposed_months: i32,
    pub criminal_history_points: i32,
    pub category_adjustment: bool,
}

/// Substantial assistance details</
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubstantialAssistance {
    pub motion_filed: bool,
    pub sealed: bool,
    pub departure_granted: bool,
    pub departure_extent_months: i32,
    pub cooperation_details_sealed: bool,
    pub safety_valve_applicable: bool,
}

/// Guidelines calculator request
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GuidelinesCalculation {
    pub offense_statute: String,
    pub drug_weight_grams: Option<f64>,
    pub loss_amount: Option<f64>,
    pub victims_count: Option<i32>,
    pub weapon_involved: bool,
    pub bodily_injury: Option<InjuryLevel>,
    pub defendant_role: DefendantRole,
    pub acceptance_of_responsibility: bool,
    pub obstruction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum InjuryLevel {
    None,
    MinorBodily,
    SeriousBodily,
    PermanentLifeThreatening,
    Death,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum DefendantRole {
    MinimalParticipant,
    MinorParticipant,
    Average,
    Manager,
    Leader,
}

/// Sentencing statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SentencingStatistics {
    pub total_cases: i32,
    pub within_guidelines: i32,
    pub upward_departures: i32,
    pub downward_departures: i32,
    pub upward_variances: i32,
    pub downward_variances: i32,
    pub government_sponsored_below: i32,
    pub substantial_assistance: i32,
    pub average_sentence_months: f64,
    pub median_sentence_months: f64,
    pub trial_penalty_percentage: f64,
}

impl Sentencing {
    /// Create new sentencing record
    pub fn new(case_id: String, defendant_id: String, judge_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            case_id,
            defendant_id,
            judge_id,
            offense_level: OffenseLevel {
                base_offense_level: 0,
                specific_offense_characteristics: Vec::new(),
                cross_references: Vec::new(),
                adjusted_offense_level: 0,
                acceptance_of_responsibility: 0,
                obstruction_enhancement: 0,
                role_adjustment: 0,
                multiple_count_adjustment: None,
                final_offense_level: 0,
            },
            criminal_history: CriminalHistory {
                category: CriminalHistoryCategory::I,
                points: 0,
                prior_sentences: Vec::new(),
                status_points: 0,
                recency_points: 0,
                career_offender: false,
                armed_career_criminal: false,
            },
            guidelines_range: GuidelinesRange {
                minimum_months: 0,
                maximum_months: 0,
                zone: Zone::A,
                mandatory_minimum: None,
                statutory_maximum: None,
            },
            adjustments: Vec::new(),
            departures: Vec::new(),
            variance: None,
            statutory_factors: StatutoryFactors {
                nature_and_circumstances: String::new(),
                defendant_history: String::new(),
                seriousness_of_offense: String::new(),
                respect_for_law: String::new(),
                just_punishment: String::new(),
                deterrence: String::new(),
                protect_public: String::new(),
                defendant_needs: String::new(),
                victim_impact: String::new(),
                avoid_disparities: String::new(),
                restitution_needed: false,
            },
            imposed_sentence: None,
            supervised_release: None,
            special_conditions: Vec::new(),
            bop_designation: None,
            rdap_eligibility: false,
            presentence_report_date: None,
            sentencing_date: None,
            judgment_date: None,
            appeal_waiver: false,
            substantial_assistance: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Calculate final offense level
    pub fn calculate_final_offense_level(&mut self) {
        let mut level = self.offense_level.base_offense_level;

        // Add specific offense characteristics
        for soc in &self.offense_level.specific_offense_characteristics {
            level += soc.adjustment;
        }

        self.offense_level.adjusted_offense_level = level;

        // Apply adjustments
        level += self.offense_level.acceptance_of_responsibility;
        level += self.offense_level.obstruction_enhancement;
        level += self.offense_level.role_adjustment;

        // Multiple count adjustment
        if let Some(ref mc) = self.offense_level.multiple_count_adjustment {
            level += mc.adjustment;
        }

        // Apply departures
        for departure in &self.departures {
            match departure.direction {
                DepartureDirection::Upward => level += departure.levels,
                DepartureDirection::Downward => level -= departure.levels,
            }
        }

        self.offense_level.final_offense_level = level.max(1);
    }

    /// Determine criminal history category from points
    pub fn calculate_criminal_history_category(&mut self) {
        let total_points = self.criminal_history.points
            + self.criminal_history.status_points
            + self.criminal_history.recency_points;

        self.criminal_history.category = match total_points {
            0..=1 => CriminalHistoryCategory::I,
            2..=3 => CriminalHistoryCategory::II,
            4..=6 => CriminalHistoryCategory::III,
            7..=9 => CriminalHistoryCategory::IV,
            10..=12 => CriminalHistoryCategory::V,
            _ => CriminalHistoryCategory::VI,
        };
    }

    /// Look up guidelines range from sentencing table
    pub fn lookup_guidelines_range(&mut self) {
        // Simplified sentencing table lookup
        // In reality, this would use the full USSC sentencing table
        let offense_level = self.offense_level.final_offense_level;
        let category = &self.criminal_history.category;

        // This is a simplified calculation - actual table is much more complex
        let base_months = match offense_level {
            1..=8 => 0,
            9..=11 => 4,
            12..=13 => 10,
            14..=15 => 15,
            16..=17 => 21,
            18..=19 => 27,
            20..=21 => 33,
            22..=23 => 41,
            24..=25 => 51,
            26..=27 => 63,
            28..=29 => 78,
            30..=31 => 97,
            32..=33 => 121,
            34..=35 => 151,
            36..=37 => 188,
            38..=39 => 235,
            40..=41 => 292,
            42..=43 => 360,
            _ => 360,
        };

        // Adjust for criminal history
        let multiplier = match category {
            CriminalHistoryCategory::I => 1.0,
            CriminalHistoryCategory::II => 1.1,
            CriminalHistoryCategory::III => 1.2,
            CriminalHistoryCategory::IV => 1.3,
            CriminalHistoryCategory::V => 1.4,
            CriminalHistoryCategory::VI => 1.5,
        };

        let min = (base_months as f32 * multiplier) as i32;
        let max = (min as f32 * 1.25) as i32; // Typically 25% range

        self.guidelines_range.minimum_months = min;
        self.guidelines_range.maximum_months = max;

        // Determine zone
        self.guidelines_range.zone = match min {
            0..=6 => Zone::A,
            7..=12 => Zone::B,
            13..=18 => Zone::C,
            _ => Zone::D,
        };
    }

    /// Check if eligible for safety valve
    pub fn is_safety_valve_eligible(&self) -> bool {
        // 18 U.S.C. § 3553(f) criteria
        !self.criminal_history.career_offender
            && !self.criminal_history.armed_career_criminal
            && self.criminal_history.points <= 1
            && self.criminal_history.prior_sentences.iter()
                .filter(|s| s.violence_involved).count() == 0
    }
}

/// Request DTO for creating a new sentencing
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSentencingRequest {
    pub case_id: String,
    pub defendant_id: String,
    pub judge_id: String,
}