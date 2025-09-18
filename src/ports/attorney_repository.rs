//! Attorney Repository Port
//!
//! Defines the interface for attorney and party data persistence.

use crate::domain::attorney::{
    Attorney, AttorneyStatus, Party, PartyStatus, AttorneyRepresentation,
    ConflictCheck, ServiceRecord, AttorneyMetrics, BarAdmission, FederalAdmission,
    ProHacViceAdmission, CJAAppointment, ECFRegistration, DisciplinaryAction
};
use anyhow::Result;

/// Repository for attorney and party management
pub trait AttorneyRepository {
    // Attorney Management
    fn save_attorney(&self, attorney: Attorney) -> Result<Attorney>;
    fn find_attorney_by_id(&self, id: &str) -> Result<Option<Attorney>>;
    fn find_attorney_by_bar_number(&self, bar_number: &str) -> Result<Option<Attorney>>;
    fn find_attorneys_by_firm(&self, firm_name: &str) -> Result<Vec<Attorney>>;
    fn find_attorneys_by_status(&self, status: AttorneyStatus) -> Result<Vec<Attorney>>;
    fn find_all_attorneys(&self) -> Result<Vec<Attorney>>;
    fn update_attorney(&self, attorney: Attorney) -> Result<Attorney>;
    fn delete_attorney(&self, id: &str) -> Result<()>;
    fn search_attorneys(&self, query: &str) -> Result<Vec<Attorney>>;

    // Bar Admissions
    fn add_bar_admission(&self, attorney_id: &str, admission: BarAdmission) -> Result<()>;
    fn remove_bar_admission(&self, attorney_id: &str, state: &str) -> Result<()>;
    fn find_attorneys_by_bar_state(&self, state: &str) -> Result<Vec<Attorney>>;

    // Federal Court Admissions
    fn add_federal_admission(&self, attorney_id: &str, admission: FederalAdmission) -> Result<()>;
    fn remove_federal_admission(&self, attorney_id: &str, court: &str) -> Result<()>;
    fn find_attorneys_admitted_to_court(&self, court: &str) -> Result<Vec<Attorney>>;

    // Pro Hac Vice
    fn add_pro_hac_vice(&self, attorney_id: &str, admission: ProHacViceAdmission) -> Result<()>;
    fn update_pro_hac_vice_status(&self, attorney_id: &str, case_id: &str, status: String) -> Result<()>;
    fn find_active_pro_hac_vice(&self) -> Result<Vec<ProHacViceAdmission>>;
    fn find_pro_hac_vice_by_case(&self, case_id: &str) -> Result<Vec<ProHacViceAdmission>>;

    // CJA Panel Management
    fn add_to_cja_panel(&self, attorney_id: &str, district: &str) -> Result<()>;
    fn remove_from_cja_panel(&self, attorney_id: &str, district: &str) -> Result<()>;
    fn find_cja_panel_attorneys(&self, district: &str) -> Result<Vec<Attorney>>;
    fn add_cja_appointment(&self, attorney_id: &str, appointment: CJAAppointment) -> Result<()>;
    fn find_cja_appointments_by_attorney(&self, attorney_id: &str) -> Result<Vec<CJAAppointment>>;
    fn find_pending_cja_vouchers(&self) -> Result<Vec<CJAAppointment>>;

    // ECF Registration
    fn update_ecf_registration(&self, attorney_id: &str, registration: ECFRegistration) -> Result<()>;
    fn find_attorneys_with_ecf_access(&self) -> Result<Vec<Attorney>>;
    fn revoke_ecf_access(&self, attorney_id: &str) -> Result<()>;

    // Discipline
    fn add_disciplinary_action(&self, attorney_id: &str, action: DisciplinaryAction) -> Result<()>;
    fn find_disciplinary_history(&self, attorney_id: &str) -> Result<Vec<DisciplinaryAction>>;
    fn find_attorneys_with_discipline(&self) -> Result<Vec<Attorney>>;

    // Party Management
    fn save_party(&self, party: Party) -> Result<Party>;
    fn find_party_by_id(&self, id: &str) -> Result<Option<Party>>;
    fn find_parties_by_case(&self, case_id: &str) -> Result<Vec<Party>>;
    fn find_parties_by_attorney(&self, attorney_id: &str) -> Result<Vec<Party>>;
    fn update_party(&self, party: Party) -> Result<Party>;
    fn delete_party(&self, id: &str) -> Result<()>;
    fn update_party_status(&self, party_id: &str, status: PartyStatus) -> Result<()>;
    fn find_unrepresented_parties(&self) -> Result<Vec<Party>>;

    // Attorney Representation
    fn add_representation(&self, representation: AttorneyRepresentation) -> Result<()>;
    fn end_representation(&self, representation_id: &str, reason: Option<String>) -> Result<()>;
    fn find_representation_by_id(&self, id: &str) -> Result<Option<AttorneyRepresentation>>;
    fn find_active_representations(&self, attorney_id: &str) -> Result<Vec<AttorneyRepresentation>>;
    fn find_representations_by_case(&self, case_id: &str) -> Result<Vec<AttorneyRepresentation>>;
    fn substitute_attorney(&self, old_attorney_id: &str, new_attorney_id: &str, case_id: &str) -> Result<()>;

    // Service Records
    fn save_service_record(&self, record: ServiceRecord) -> Result<()>;
    fn find_service_records_by_document(&self, document_id: &str) -> Result<Vec<ServiceRecord>>;
    fn find_service_records_by_party(&self, party_id: &str) -> Result<Vec<ServiceRecord>>;
    fn mark_service_completed(&self, record_id: &str) -> Result<()>;

    // Conflict Checking
    fn save_conflict_check(&self, check: ConflictCheck) -> Result<()>;
    fn find_conflict_checks_by_attorney(&self, attorney_id: &str) -> Result<Vec<ConflictCheck>>;
    fn find_conflicts_for_parties(&self, attorney_id: &str, party_names: Vec<String>) -> Result<Vec<ConflictCheck>>;
    fn clear_conflict(&self, check_id: &str, waiver_obtained: bool) -> Result<()>;

    // Attorney Metrics
    fn calculate_attorney_metrics(&self, attorney_id: &str, start_date: &str, end_date: &str) -> Result<AttorneyMetrics>;
    fn get_attorney_win_rate(&self, attorney_id: &str) -> Result<f64>;
    fn get_attorney_case_count(&self, attorney_id: &str) -> Result<i32>;
    fn get_top_performing_attorneys(&self, limit: usize) -> Result<Vec<(Attorney, AttorneyMetrics)>>;

    // Bulk Operations
    fn bulk_update_attorney_status(&self, attorney_ids: Vec<String>, status: AttorneyStatus) -> Result<()>;
    fn bulk_add_to_service_list(&self, document_id: &str, party_ids: Vec<String>) -> Result<()>;
    fn migrate_representations(&self, from_attorney_id: &str, to_attorney_id: &str) -> Result<()>;
}