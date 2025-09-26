//! Spin KV Attorney Repository Adapter
//!
//! Implements attorney and party persistence using Spin's key-value store.

use crate::adapters::store_utils::open_validated_store;
use crate::domain::attorney::{
    Attorney, AttorneyStatus, Party, PartyStatus, AttorneyRepresentation,
    ConflictCheck, ServiceRecord, AttorneyMetrics, BarAdmission, FederalAdmission,
    ProHacViceAdmission, CJAAppointment, ECFRegistration, DisciplinaryAction,
    ServiceMethod, VoucherStatus
};
use crate::ports::attorney_repository::AttorneyRepository;
use anyhow::Result;
use chrono::Utc;
use serde_json;
use spin_sdk::key_value::Store;
use uuid::Uuid;

pub struct SpinKvAttorneyRepository {
    store: Store,
}

impl SpinKvAttorneyRepository {
    /// Create repository with specific store name for multi-tenancy
    /// MUST fail if the store doesn't exist to prevent data mixing
    pub fn with_store(store_name: String) -> Self {
        let store = open_validated_store(&store_name)
            .expect(&format!("Failed to open store: {}. In development, use 'default' as tenant ID", store_name));
        Self { store }
    }

    fn get_key(&self, prefix: &str, id: &str) -> String {
        // Simple key format - tenant isolation is handled by separate physical stores
        // No need for tenant prefix in keys since each store is isolated
        format!("{}:{}", prefix, id)
    }

    fn save_json<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value)?;
        self.store.set(key, &json)?;
        Ok(())
    }

    fn get_json<T: for<'de> serde::Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        match self.store.get(key) {
            Ok(Some(bytes)) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("KV store error: {}", e)),
        }
    }

    fn list_with_prefix<T: for<'de> serde::Deserialize<'de>>(&self, prefix: &str) -> Result<Vec<T>> {
        let keys = self.store.get_keys()?;
        let mut results = Vec::new();

        for key in keys.iter() {
            if key.starts_with(prefix) {
                if let Some(value) = self.get_json::<T>(key)? {
                    results.push(value);
                }
            }
        }

        Ok(results)
    }
}

impl AttorneyRepository for SpinKvAttorneyRepository {
    fn save_attorney(&self, mut attorney: Attorney) -> Result<Attorney> {
        if attorney.id.is_empty() {
            attorney.id = Uuid::new_v4().to_string();
        }
        attorney.updated_at = Utc::now();

        let key = self.get_key("attorney", &attorney.id);
        self.save_json(&key, &attorney)?;

        // Update indexes
        let bar_key = format!("idx:bar:{}", attorney.bar_number);
        self.store.set(&bar_key, attorney.id.as_bytes())?;

        if let Some(ref firm) = attorney.firm_name {
            let firm_key = format!("idx:firm:{}:{}", firm, attorney.id);
            self.store.set(&firm_key, attorney.id.as_bytes())?;
        }

        Ok(attorney)
    }

    fn find_attorney_by_id(&self, id: &str) -> Result<Option<Attorney>> {
        let key = self.get_key("attorney", id);
        self.get_json(&key)
    }

    fn find_attorney_by_bar_number(&self, bar_number: &str) -> Result<Option<Attorney>> {
        let idx_key = format!("idx:bar:{}", bar_number);

        match self.store.get(&idx_key) {
            Ok(Some(id_bytes)) => {
                let id = String::from_utf8(id_bytes)?;
                self.find_attorney_by_id(&id)
            }
            _ => Ok(None)
        }
    }

    fn find_attorneys_by_firm(&self, firm_name: &str) -> Result<Vec<Attorney>> {
        let prefix = format!("idx:firm:{}", firm_name);
        let keys = self.store.get_keys()?;
        let mut attorneys = Vec::new();

        for key in keys.iter() {
            if key.starts_with(&prefix) {
                if let Some(id_bytes) = self.store.get(key)? {
                    let id = String::from_utf8(id_bytes)?;
                    if let Some(attorney) = self.find_attorney_by_id(&id)? {
                        attorneys.push(attorney);
                    }
                }
            }
        }

        Ok(attorneys)
    }

    fn find_attorneys_by_status(&self, status: AttorneyStatus) -> Result<Vec<Attorney>> {
        let attorneys = self.list_with_prefix::<Attorney>("attorney:")?;
        Ok(attorneys.into_iter()
            .filter(|a| a.status == status)
            .collect())
    }

    fn find_all_attorneys(&self) -> Result<Vec<Attorney>> {
        self.list_with_prefix("attorney:")
    }

    fn update_attorney(&self, mut attorney: Attorney) -> Result<Attorney> {
        if attorney.id.is_empty() {
            return Err(anyhow::anyhow!("Cannot update attorney without ID"));
        }

        // Clean up old indexes if the attorney exists and bar number changed
        if let Some(existing) = self.find_attorney_by_id(&attorney.id)? {
            if existing.bar_number != attorney.bar_number {
                // Delete the old bar number index
                let old_bar_key = format!("idx:bar:{}", existing.bar_number);
                self.store.delete(&old_bar_key)?;
            }

            // Handle firm index changes
            if existing.firm_name != attorney.firm_name {
                if let Some(ref old_firm) = existing.firm_name {
                    let old_firm_key = format!("idx:firm:{}:{}", old_firm, attorney.id);
                    self.store.delete(&old_firm_key)?;
                }
            }
        }

        // Update the attorney (this will create new indexes)
        let key = self.get_key("attorney", &attorney.id);
        self.save_json(&key, &attorney)?;

        // Create bar number index
        let bar_key = format!("idx:bar:{}", attorney.bar_number);
        self.store.set(&bar_key, attorney.id.as_bytes())?;

        // Create firm index if needed
        if let Some(ref firm) = attorney.firm_name {
            let firm_key = format!("idx:firm:{}:{}", firm, attorney.id);
            self.store.set(&firm_key, attorney.id.as_bytes())?;
        }

        Ok(attorney)
    }

    fn delete_attorney(&self, id: &str) -> Result<()> {
        // First get the attorney to clean up indexes
        if let Some(attorney) = self.find_attorney_by_id(id)? {
            // Delete bar number index
            let bar_key = format!("idx:bar:{}", attorney.bar_number);
            self.store.delete(&bar_key)?;

            // Delete firm index if exists
            if let Some(ref firm) = attorney.firm_name {
                let firm_key = format!("idx:firm:{}:{}", firm, attorney.id);
                self.store.delete(&firm_key)?;
            }
        }

        // Delete the attorney record
        let key = self.get_key("attorney", id);
        self.store.delete(&key)?;
        Ok(())
    }

    fn search_attorneys(&self, query: &str) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        let query_lower = query.to_lowercase();

        Ok(attorneys.into_iter()
            .filter(|a| {
                a.first_name.to_lowercase().contains(&query_lower) ||
                a.last_name.to_lowercase().contains(&query_lower) ||
                a.bar_number.to_lowercase().contains(&query_lower) ||
                a.email.to_lowercase().contains(&query_lower) ||
                a.firm_name.as_ref().map_or(false, |f| f.to_lowercase().contains(&query_lower))
            })
            .collect())
    }

    fn add_bar_admission(&self, attorney_id: &str, admission: BarAdmission) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.bar_admissions.push(admission);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn remove_bar_admission(&self, attorney_id: &str, state: &str) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.bar_admissions.retain(|a| a.state != state);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_attorneys_by_bar_state(&self, state: &str) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        Ok(attorneys.into_iter()
            .filter(|a| a.bar_admissions.iter().any(|adm| adm.state == state))
            .collect())
    }

    fn add_federal_admission(&self, attorney_id: &str, admission: FederalAdmission) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.federal_admissions.push(admission);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn remove_federal_admission(&self, attorney_id: &str, court: &str) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.federal_admissions.retain(|a| a.court != court);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_attorneys_admitted_to_court(&self, court: &str) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        Ok(attorneys.into_iter()
            .filter(|a| a.federal_admissions.iter().any(|adm| adm.court == court))
            .collect())
    }

    fn add_pro_hac_vice(&self, attorney_id: &str, admission: ProHacViceAdmission) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.pro_hac_vice_admissions.push(admission);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn update_pro_hac_vice_status(&self, attorney_id: &str, case_id: &str, status: String) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            for phv in &mut attorney.pro_hac_vice_admissions {
                if phv.case_id == case_id {
                    // Parse status string to enum
                    phv.status = serde_json::from_str(&format!("\"{}\"", status))?;
                    break;
                }
            }
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_active_pro_hac_vice(&self) -> Result<Vec<ProHacViceAdmission>> {
        let attorneys = self.find_all_attorneys()?;
        let mut admissions = Vec::new();

        for attorney in attorneys {
            for phv in attorney.pro_hac_vice_admissions {
                if matches!(phv.status, crate::domain::attorney::ProHacViceStatus::Granted) {
                    admissions.push(phv);
                }
            }
        }

        Ok(admissions)
    }

    fn find_pro_hac_vice_by_case(&self, case_id: &str) -> Result<Vec<ProHacViceAdmission>> {
        let attorneys = self.find_all_attorneys()?;
        let mut admissions = Vec::new();

        for attorney in attorneys {
            for phv in attorney.pro_hac_vice_admissions {
                if phv.case_id == case_id {
                    admissions.push(phv);
                }
            }
        }

        Ok(admissions)
    }

    fn add_to_cja_panel(&self, attorney_id: &str, district: &str) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.cja_panel_member = true;
            if !attorney.cja_panel_districts.contains(&district.to_string()) {
                attorney.cja_panel_districts.push(district.to_string());
            }
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn remove_from_cja_panel(&self, attorney_id: &str, district: &str) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.cja_panel_districts.retain(|d| d != district);
            if attorney.cja_panel_districts.is_empty() {
                attorney.cja_panel_member = false;
            }
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_cja_panel_attorneys(&self, district: &str) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        Ok(attorneys.into_iter()
            .filter(|a| a.cja_panel_member && a.cja_panel_districts.contains(&district.to_string()))
            .collect())
    }

    fn add_cja_appointment(&self, attorney_id: &str, appointment: CJAAppointment) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.cja_appointments.push(appointment);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_cja_appointments_by_attorney(&self, attorney_id: &str) -> Result<Vec<CJAAppointment>> {
        if let Some(attorney) = self.find_attorney_by_id(attorney_id)? {
            Ok(attorney.cja_appointments)
        } else {
            Ok(Vec::new())
        }
    }

    fn find_pending_cja_vouchers(&self) -> Result<Vec<CJAAppointment>> {
        let attorneys = self.find_all_attorneys()?;
        let mut appointments = Vec::new();

        for attorney in attorneys {
            for appt in attorney.cja_appointments {
                if matches!(appt.voucher_status, VoucherStatus::Submitted | VoucherStatus::UnderReview) {
                    appointments.push(appt);
                }
            }
        }

        Ok(appointments)
    }

    fn update_ecf_registration(&self, attorney_id: &str, registration: ECFRegistration) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.ecf_registration = Some(registration);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_attorneys_with_ecf_access(&self) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        Ok(attorneys.into_iter()
            .filter(|a| a.ecf_registration.as_ref().map_or(false, |ecf| ecf.active))
            .collect())
    }

    fn revoke_ecf_access(&self, attorney_id: &str) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            if let Some(ref mut ecf) = attorney.ecf_registration {
                ecf.active = false;
            }
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn add_disciplinary_action(&self, attorney_id: &str, action: DisciplinaryAction) -> Result<()> {
        if let Some(mut attorney) = self.find_attorney_by_id(attorney_id)? {
            attorney.discipline_history.push(action);
            self.save_attorney(attorney)?;
        }
        Ok(())
    }

    fn find_disciplinary_history(&self, attorney_id: &str) -> Result<Vec<DisciplinaryAction>> {
        if let Some(attorney) = self.find_attorney_by_id(attorney_id)? {
            Ok(attorney.discipline_history)
        } else {
            Ok(Vec::new())
        }
    }

    fn find_attorneys_with_discipline(&self) -> Result<Vec<Attorney>> {
        let attorneys = self.find_all_attorneys()?;
        Ok(attorneys.into_iter()
            .filter(|a| !a.discipline_history.is_empty())
            .collect())
    }

    fn save_party(&self, mut party: Party) -> Result<Party> {
        if party.id.is_empty() {
            party.id = Uuid::new_v4().to_string();
        }
        party.updated_at = Utc::now();

        let key = self.get_key("party", &party.id);
        self.save_json(&key, &party)?;

        // Index by case
        let case_key = format!("idx:party:case:{}:{}", party.case_id, party.id);
        self.store.set(&case_key, party.id.as_bytes())?;

        Ok(party)
    }

    fn find_party_by_id(&self, id: &str) -> Result<Option<Party>> {
        let key = self.get_key("party", id);
        self.get_json(&key)
    }

    fn find_parties_by_case(&self, case_id: &str) -> Result<Vec<Party>> {
        let prefix = format!("idx:party:case:{}", case_id);
        let keys = self.store.get_keys()?;
        let mut parties = Vec::new();

        for key in keys.iter() {
            if key.starts_with(&prefix) {
                if let Some(id_bytes) = self.store.get(key)? {
                    let id = String::from_utf8(id_bytes)?;
                    if let Some(party) = self.find_party_by_id(&id)? {
                        parties.push(party);
                    }
                }
            }
        }

        Ok(parties)
    }

    fn find_parties_by_attorney(&self, attorney_id: &str) -> Result<Vec<Party>> {
        let parties = self.list_with_prefix::<Party>("party:")?;
        Ok(parties.into_iter()
            .filter(|p| p.attorneys.iter().any(|rep| rep.attorney_id == attorney_id))
            .collect())
    }

    fn update_party(&self, party: Party) -> Result<Party> {
        if party.id.is_empty() {
            return Err(anyhow::anyhow!("Cannot update party without ID"));
        }
        self.save_party(party)
    }

    fn delete_party(&self, id: &str) -> Result<()> {
        // Clean up indexes
        if let Some(party) = self.find_party_by_id(id)? {
            let case_key = format!("idx:party:case:{}:{}", party.case_id, party.id);
            self.store.delete(&case_key)?;
        }

        let key = self.get_key("party", id);
        self.store.delete(&key)?;
        Ok(())
    }

    fn update_party_status(&self, party_id: &str, status: PartyStatus) -> Result<()> {
        if let Some(mut party) = self.find_party_by_id(party_id)? {
            party.status = status.clone();
            if matches!(status, PartyStatus::Terminated | PartyStatus::Dismissed | PartyStatus::Settled) {
                party.terminated_date = Some(Utc::now());
            }
            self.save_party(party)?;
        }
        Ok(())
    }

    fn find_unrepresented_parties(&self) -> Result<Vec<Party>> {
        let parties = self.list_with_prefix::<Party>("party:")?;
        Ok(parties.into_iter()
            .filter(|p| !p.represented && !p.pro_se)
            .collect())
    }

    fn add_representation(&self, mut representation: AttorneyRepresentation) -> Result<()> {
        if representation.id.is_empty() {
            representation.id = Uuid::new_v4().to_string();
        }

        let key = self.get_key("representation", &representation.id);
        self.save_json(&key, &representation)?;

        // Update party's attorney list
        if let Some(mut party) = self.find_party_by_id(&representation.party_id)? {
            party.attorneys.push(representation.clone());
            party.represented = true;
            self.save_party(party)?;
        }

        // Index by attorney
        let att_key = format!("idx:rep:attorney:{}:{}", representation.attorney_id, representation.id);
        self.store.set(&att_key, representation.id.as_bytes())?;

        Ok(())
    }

    fn end_representation(&self, representation_id: &str, reason: Option<String>) -> Result<()> {
        let key = self.get_key("representation", representation_id);
        if let Some(mut rep) = self.get_json::<AttorneyRepresentation>(&key)? {
            rep.end_date = Some(Utc::now());
            if let Some(reason_str) = reason {
                rep.withdrawal_reason = serde_json::from_str(&format!("\"{}\"", reason_str)).ok();
            }
            self.save_json(&key, &rep)?;

            // Update party's representation status
            if let Some(mut party) = self.find_party_by_id(&rep.party_id)? {
                party.attorneys.retain(|r| r.id != representation_id);
                if party.attorneys.is_empty() {
                    party.represented = false;
                }
                self.save_party(party)?;
            }
        }
        Ok(())
    }

    fn find_representation_by_id(&self, id: &str) -> Result<Option<AttorneyRepresentation>> {
        let key = self.get_key("representation", id);
        self.get_json(&key)
    }

    fn find_active_representations(&self, attorney_id: &str) -> Result<Vec<AttorneyRepresentation>> {
        let prefix = format!("idx:rep:attorney:{}", attorney_id);
        let keys = self.store.get_keys()?;
        let mut reps = Vec::new();

        for key in keys.iter() {
            if key.starts_with(&prefix) {
                if let Some(id_bytes) = self.store.get(key)? {
                    let id = String::from_utf8(id_bytes)?;
                    if let Some(rep) = self.find_representation_by_id(&id)? {
                        if rep.end_date.is_none() {
                            reps.push(rep);
                        }
                    }
                }
            }
        }

        Ok(reps)
    }

    fn find_representations_by_case(&self, case_id: &str) -> Result<Vec<AttorneyRepresentation>> {
        let representations = self.list_with_prefix::<AttorneyRepresentation>("representation:")?;
        Ok(representations.into_iter()
            .filter(|r| r.case_id == case_id)
            .collect())
    }

    fn substitute_attorney(&self, old_attorney_id: &str, new_attorney_id: &str, case_id: &str) -> Result<()> {
        let representations = self.find_representations_by_case(case_id)?;

        for rep in representations {
            if rep.attorney_id == old_attorney_id && rep.end_date.is_none() {
                // End old representation
                self.end_representation(&rep.id, Some("Substitution".to_string()))?;

                // Create new representation
                let new_rep = AttorneyRepresentation {
                    id: Uuid::new_v4().to_string(),
                    attorney_id: new_attorney_id.to_string(),
                    party_id: rep.party_id.clone(),
                    case_id: rep.case_id.clone(),
                    representation_type: rep.representation_type.clone(),
                    start_date: Utc::now(),
                    end_date: None,
                    lead_counsel: rep.lead_counsel,
                    local_counsel: rep.local_counsel,
                    limited_appearance: rep.limited_appearance,
                    scope_of_representation: rep.scope_of_representation.clone(),
                    withdrawal_reason: None,
                    court_appointed: rep.court_appointed,
                    cja_appointment_id: rep.cja_appointment_id.clone(),
                };
                self.add_representation(new_rep)?;
            }
        }

        Ok(())
    }

    fn save_service_record(&self, mut record: ServiceRecord) -> Result<()> {
        if record.id.is_empty() {
            record.id = Uuid::new_v4().to_string();
        }

        let key = self.get_key("service", &record.id);
        self.save_json(&key, &record)?;

        // Index by document
        let doc_key = format!("idx:service:doc:{}:{}", record.document_id, record.id);
        self.store.set(&doc_key, record.id.as_bytes())?;

        Ok(())
    }

    fn find_service_records_by_document(&self, document_id: &str) -> Result<Vec<ServiceRecord>> {
        let prefix = format!("idx:service:doc:{}", document_id);
        let keys = self.store.get_keys()?;
        let mut records = Vec::new();

        for key in keys.iter() {
            if key.starts_with(&prefix) {
                if let Some(id_bytes) = self.store.get(key)? {
                    let id = String::from_utf8(id_bytes)?;
                    let rec_key = self.get_key("service", &id);
                    if let Some(record) = self.get_json::<ServiceRecord>(&rec_key)? {
                        records.push(record);
                    }
                }
            }
        }

        Ok(records)
    }

    fn find_service_records_by_party(&self, party_id: &str) -> Result<Vec<ServiceRecord>> {
        let records = self.list_with_prefix::<ServiceRecord>("service:")?;
        Ok(records.into_iter()
            .filter(|r| r.party_id == party_id)
            .collect())
    }

    fn mark_service_completed(&self, record_id: &str) -> Result<()> {
        let key = self.get_key("service", record_id);
        if let Some(mut record) = self.get_json::<ServiceRecord>(&key)? {
            record.successful = true;
            record.proof_of_service_filed = true;
            self.save_json(&key, &record)?;
        }
        Ok(())
    }

    fn save_conflict_check(&self, mut check: ConflictCheck) -> Result<()> {
        if check.id.is_empty() {
            check.id = Uuid::new_v4().to_string();
        }

        let key = self.get_key("conflict", &check.id);
        self.save_json(&key, &check)?;

        // Index by attorney
        let att_key = format!("idx:conflict:attorney:{}:{}", check.attorney_id, check.id);
        self.store.set(&att_key, check.id.as_bytes())?;

        Ok(())
    }

    fn find_conflict_checks_by_attorney(&self, attorney_id: &str) -> Result<Vec<ConflictCheck>> {
        let prefix = format!("idx:conflict:attorney:{}", attorney_id);
        let keys = self.store.get_keys()?;
        let mut checks = Vec::new();

        for key in keys.iter() {
            if key.starts_with(&prefix) {
                if let Some(id_bytes) = self.store.get(key)? {
                    let id = String::from_utf8(id_bytes)?;
                    let check_key = self.get_key("conflict", &id);
                    if let Some(check) = self.get_json::<ConflictCheck>(&check_key)? {
                        checks.push(check);
                    }
                }
            }
        }

        Ok(checks)
    }

    fn find_conflicts_for_parties(&self, attorney_id: &str, party_names: Vec<String>) -> Result<Vec<ConflictCheck>> {
        let checks = self.find_conflict_checks_by_attorney(attorney_id)?;
        let party_names_lower: Vec<String> = party_names.iter().map(|n| n.to_lowercase()).collect();

        Ok(checks.into_iter()
            .filter(|check| {
                check.party_names.iter().any(|name| {
                    let name_lower = name.to_lowercase();
                    party_names_lower.iter().any(|pn| name_lower.contains(pn) || pn.contains(&name_lower))
                }) ||
                check.adverse_parties.iter().any(|name| {
                    let name_lower = name.to_lowercase();
                    party_names_lower.iter().any(|pn| name_lower.contains(pn) || pn.contains(&name_lower))
                })
            })
            .collect())
    }

    fn clear_conflict(&self, check_id: &str, waiver_obtained: bool) -> Result<()> {
        let key = self.get_key("conflict", check_id);
        if let Some(mut check) = self.get_json::<ConflictCheck>(&key)? {
            check.cleared = true;
            check.waiver_obtained = waiver_obtained;
            self.save_json(&key, &check)?;
        }
        Ok(())
    }

    fn calculate_attorney_metrics(&self, attorney_id: &str, start_date: &str, end_date: &str) -> Result<AttorneyMetrics> {
        let attorney = self.find_attorney_by_id(attorney_id)?
            .ok_or_else(|| anyhow::anyhow!("Attorney not found"))?;

        // Parse dates
        let period_start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")?
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let period_end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")?
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        // Get all representations for this attorney
        let all_reps = self.list_with_prefix::<AttorneyRepresentation>("representation:")?;
        let attorney_reps: Vec<_> = all_reps.into_iter()
            .filter(|r| r.attorney_id == attorney_id)
            .filter(|r| r.start_date >= period_start && r.start_date <= period_end)
            .collect();

        let total_cases = attorney_reps.len() as i32;
        let active_cases = attorney_reps.iter().filter(|r| r.end_date.is_none()).count() as i32;
        let completed_cases = attorney_reps.iter().filter(|r| r.end_date.is_some()).count() as i32;

        // Calculate CJA metrics
        let cja_appointments = attorney.cja_appointments.iter()
            .filter(|a| a.appointment_date >= period_start && a.appointment_date <= period_end)
            .count() as i32;

        let cja_hours_billed = attorney.cja_appointments.iter()
            .filter(|a| a.appointment_date >= period_start && a.appointment_date <= period_end)
            .map(|a| a.hours_claimed)
            .sum();

        let cja_amount_approved = attorney.cja_appointments.iter()
            .filter(|a| a.appointment_date >= period_start && a.appointment_date <= period_end)
            .filter_map(|a| a.amount_approved)
            .sum();

        Ok(AttorneyMetrics {
            attorney_id: attorney_id.to_string(),
            period_start,
            period_end,
            total_cases,
            active_cases,
            completed_cases,
            cases_won: 0, // Would need case outcome data
            cases_lost: 0,
            cases_settled: 0,
            cases_dismissed: 0,
            total_filings: 0, // Would need filing data
            motions_filed: 0,
            motions_granted: 0,
            appeals_filed: 0,
            appeals_won: 0,
            avg_case_duration_days: attorney.avg_case_duration_days.unwrap_or(0) as f64,
            avg_response_time_hours: 24.0, // Default
            cja_appointments,
            cja_hours_billed,
            cja_amount_approved,
            missed_deadlines: 0,
            sanctions_received: 0,
            rule_violations: 0,
        })
    }

    fn get_attorney_win_rate(&self, attorney_id: &str) -> Result<f64> {
        if let Some(attorney) = self.find_attorney_by_id(attorney_id)? {
            Ok(attorney.win_rate_percentage.unwrap_or(0.0))
        } else {
            Ok(0.0)
        }
    }

    fn get_attorney_case_count(&self, attorney_id: &str) -> Result<i32> {
        if let Some(attorney) = self.find_attorney_by_id(attorney_id)? {
            Ok(attorney.cases_handled)
        } else {
            Ok(0)
        }
    }

    fn get_top_performing_attorneys(&self, limit: usize) -> Result<Vec<(Attorney, AttorneyMetrics)>> {
        let mut attorneys = self.find_all_attorneys()?;

        // Sort by win rate
        attorneys.sort_by(|a, b| {
            let a_rate = a.win_rate_percentage.unwrap_or(0.0);
            let b_rate = b.win_rate_percentage.unwrap_or(0.0);
            b_rate.partial_cmp(&a_rate).unwrap()
        });

        let mut results = Vec::new();
        let now = Utc::now();
        let year_ago = now - chrono::Duration::days(365);

        for attorney in attorneys.into_iter().take(limit) {
            let metrics = self.calculate_attorney_metrics(
                &attorney.id,
                &year_ago.format("%Y-%m-%d").to_string(),
                &now.format("%Y-%m-%d").to_string()
            )?;
            results.push((attorney, metrics));
        }

        Ok(results)
    }

    fn bulk_update_attorney_status(&self, attorney_ids: Vec<String>, status: AttorneyStatus) -> Result<()> {
        for id in attorney_ids {
            if let Some(mut attorney) = self.find_attorney_by_id(&id)? {
                attorney.status = status.clone();
                self.save_attorney(attorney)?;
            }
        }
        Ok(())
    }

    fn bulk_add_to_service_list(&self, document_id: &str, party_ids: Vec<String>) -> Result<()> {
        for party_id in party_ids {
            let record = ServiceRecord {
                id: String::new(),
                document_id: document_id.to_string(),
                party_id: party_id.clone(),
                service_date: Utc::now(),
                service_method: ServiceMethod::ECF,
                served_by: "System".to_string(),
                proof_of_service_filed: false,
                certificate_of_service: None,
                successful: false,
                attempts: 1,
                notes: Some("Bulk service".to_string()),
            };
            self.save_service_record(record)?;
        }
        Ok(())
    }

    fn migrate_representations(&self, from_attorney_id: &str, to_attorney_id: &str) -> Result<()> {
        let representations = self.find_active_representations(from_attorney_id)?;

        for rep in representations {
            // End old representation
            self.end_representation(&rep.id, Some("Attorney migration".to_string()))?;

            // Create new representation with new attorney
            let new_rep = AttorneyRepresentation {
                id: Uuid::new_v4().to_string(),
                attorney_id: to_attorney_id.to_string(),
                party_id: rep.party_id,
                case_id: rep.case_id,
                representation_type: rep.representation_type,
                start_date: Utc::now(),
                end_date: None,
                lead_counsel: rep.lead_counsel,
                local_counsel: rep.local_counsel,
                limited_appearance: rep.limited_appearance,
                scope_of_representation: rep.scope_of_representation,
                withdrawal_reason: None,
                court_appointed: rep.court_appointed,
                cja_appointment_id: rep.cja_appointment_id,
            };
            self.add_representation(new_rep)?;
        }

        Ok(())
    }
}