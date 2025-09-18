//! Repository trait for federal sentencing management

use crate::domain::sentencing::*;
use crate::ApiResult;

/// Repository trait for sentencing operations
pub trait SentencingRepository: Send + Sync {
    // Sentencing CRUD
    fn create_sentencing(&self, sentencing: Sentencing) -> ApiResult<Sentencing>;
    fn get_sentencing(&self, id: &str) -> ApiResult<Option<Sentencing>>;
    fn update_sentencing(&self, sentencing: Sentencing) -> ApiResult<Sentencing>;
    fn delete_sentencing(&self, id: &str) -> ApiResult<()>;

    // Query operations
    fn find_by_case(&self, case_id: &str) -> ApiResult<Vec<Sentencing>>;
    fn find_by_defendant(&self, defendant_id: &str) -> ApiResult<Vec<Sentencing>>;
    fn find_by_judge(&self, judge_id: &str) -> ApiResult<Vec<Sentencing>>;
    fn find_pending_sentencing(&self) -> ApiResult<Vec<Sentencing>>;
    fn find_by_date_range(&self, start: &str, end: &str) -> ApiResult<Vec<Sentencing>>;

    // Guidelines operations
    fn calculate_guidelines(&self, calculation: GuidelinesCalculation) -> ApiResult<GuidelinesRange>;
    fn get_departure_rates(&self) -> ApiResult<SentencingStatistics>;
    fn get_variance_rates(&self) -> ApiResult<SentencingStatistics>;

    // Departures and variances
    fn add_departure(&self, sentencing_id: &str, departure: Departure) -> ApiResult<Sentencing>;
    fn add_variance(&self, sentencing_id: &str, variance: Variance) -> ApiResult<Sentencing>;
    fn get_substantial_assistance_cases(&self) -> ApiResult<Vec<Sentencing>>;

    // Supervised release
    fn add_special_condition(&self, sentencing_id: &str, condition: SpecialCondition) -> ApiResult<Sentencing>;
    fn update_supervised_release(&self, sentencing_id: &str, release: SupervisedRelease) -> ApiResult<Sentencing>;
    fn find_active_supervision(&self) -> ApiResult<Vec<Sentencing>>;

    // BOP recommendations
    fn add_bop_designation(&self, sentencing_id: &str, designation: BOPDesignation) -> ApiResult<Sentencing>;
    fn get_rdap_eligible(&self) -> ApiResult<Vec<Sentencing>>;

    // Statistics and reporting
    fn get_judge_sentencing_stats(&self, judge_id: &str) -> ApiResult<SentencingStatistics>;
    fn get_district_stats(&self) -> ApiResult<SentencingStatistics>;
    fn get_offense_type_stats(&self, offense_type: &str) -> ApiResult<SentencingStatistics>;
    fn get_trial_penalty_analysis(&self) -> ApiResult<SentencingStatistics>;

    // Criminal history
    fn add_prior_sentence(&self, sentencing_id: &str, prior: PriorSentence) -> ApiResult<Sentencing>;
    fn calculate_criminal_history_points(&self, sentencing_id: &str) -> ApiResult<i32>;

    // Compliance and deadlines
    fn find_upcoming_sentencings(&self, days: i32) -> ApiResult<Vec<Sentencing>>;
    fn find_appeal_deadline_approaching(&self) -> ApiResult<Vec<Sentencing>>;
}