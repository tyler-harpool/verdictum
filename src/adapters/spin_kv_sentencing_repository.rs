//! Spin KV implementation of sentencing repository

use crate::domain::sentencing::*;
use crate::ports::sentencing_repository::SentencingRepository;
use crate::{ApiError, ApiResult};
use chrono::{Utc, NaiveDate};
use spin_sdk::key_value::{Error as KvError, Store};

pub struct SpinKvSentencingRepository {
    store: Store,
}

impl SpinKvSentencingRepository {
    /// Create repository with specific store name for multi-tenancy
    pub fn with_store(store_name: String) -> Self {
        let store = Store::open(&store_name)
            .expect(&format!("Failed to open store: {}", store_name));
        Self { store }
    }

    fn get_all_sentencings(&self) -> ApiResult<Vec<Sentencing>> {
        let keys = self.store
            .get_keys()
            .map_err(|e| ApiError::StorageError(format!("Failed to get keys: {:?}", e)))?;

        let mut sentencings = Vec::new();
        for key in keys {
            if key.starts_with("sentencing:") && !key.contains("::") {
                if let Some(bytes) = self.store.get(&key)
                    .map_err(|e| ApiError::StorageError(format!("Failed to get {}: {:?}", key, e)))? {
                    let sentencing: Sentencing = serde_json::from_slice(&bytes)
                        .map_err(|e| ApiError::SerializationError(format!("Failed to deserialize: {}", e)))?;
                    sentencings.push(sentencing);
                }
            }
        }

        Ok(sentencings)
    }

    fn save_sentencing(&self, sentencing: &Sentencing) -> ApiResult<()> {
        let key = format!("sentencing:{}", sentencing.id);
        let value = serde_json::to_vec(&sentencing)
            .map_err(|e| ApiError::SerializationError(format!("Failed to serialize: {}", e)))?;

        self.store.set(&key, &value)
            .map_err(|e| ApiError::StorageError(format!("Failed to save: {:?}", e)))?;

        // Index by case
        let case_key = format!("sentencing::case:{}", sentencing.case_id);
        self.store.set(&case_key, sentencing.id.as_bytes())
            .map_err(|e| ApiError::StorageError(format!("Failed to index by case: {:?}", e)))?;

        // Index by defendant
        let defendant_key = format!("sentencing::defendant:{}", sentencing.defendant_id);
        self.store.set(&defendant_key, sentencing.id.as_bytes())
            .map_err(|e| ApiError::StorageError(format!("Failed to index by defendant: {:?}", e)))?;

        Ok(())
    }
}

impl SentencingRepository for SpinKvSentencingRepository {
    fn create_sentencing(&self, sentencing: Sentencing) -> ApiResult<Sentencing> {
        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn get_sentencing(&self, id: &str) -> ApiResult<Option<Sentencing>> {
        let key = format!("sentencing:{}", id);

        match self.store.get(&key) {
            Ok(Some(bytes)) => {
                let sentencing: Sentencing = serde_json::from_slice(&bytes)
                    .map_err(|e| ApiError::SerializationError(format!("Failed to deserialize: {}", e)))?;
                Ok(Some(sentencing))
            }
            Ok(None) => Ok(None),
            Err(KvError::NoSuchStore) => Ok(None),
            Err(e) => Err(ApiError::StorageError(format!("Failed to get sentencing: {:?}", e)))
        }
    }

    fn update_sentencing(&self, mut sentencing: Sentencing) -> ApiResult<Sentencing> {
        sentencing.updated_at = Utc::now();
        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn delete_sentencing(&self, id: &str) -> ApiResult<()> {
        let key = format!("sentencing:{}", id);
        self.store.delete(&key)
            .map_err(|e| ApiError::StorageError(format!("Failed to delete: {:?}", e)))?;
        Ok(())
    }

    fn find_by_case(&self, case_id: &str) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.case_id == case_id)
            .collect())
    }

    fn find_by_defendant(&self, defendant_id: &str) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.defendant_id == defendant_id)
            .collect())
    }

    fn find_by_judge(&self, judge_id: &str) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.judge_id == judge_id)
            .collect())
    }

    fn find_pending_sentencing(&self) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.imposed_sentence.is_none())
            .collect())
    }

    fn find_by_date_range(&self, start: &str, end: &str) -> ApiResult<Vec<Sentencing>> {
        let start_date = NaiveDate::parse_from_str(start, "%Y-%m-%d")
            .map_err(|e| ApiError::ValidationError(format!("Invalid start date: {}", e)))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ApiError::ValidationError("Invalid time".to_string()))?
            .and_local_timezone(Utc)
            .single()
            .ok_or_else(|| ApiError::ValidationError("Invalid timezone".to_string()))?;

        let end_date = NaiveDate::parse_from_str(end, "%Y-%m-%d")
            .map_err(|e| ApiError::ValidationError(format!("Invalid end date: {}", e)))?
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| ApiError::ValidationError("Invalid time".to_string()))?
            .and_local_timezone(Utc)
            .single()
            .ok_or_else(|| ApiError::ValidationError("Invalid timezone".to_string()))?;

        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| {
                if let Some(date) = s.sentencing_date {
                    date >= start_date && date <= end_date
                } else {
                    false
                }
            })
            .collect())
    }

    fn calculate_guidelines(&self, calculation: GuidelinesCalculation) -> ApiResult<GuidelinesRange> {
        // Simplified guidelines calculation
        // In production, this would use the full USSC guidelines manual

        let mut offense_level = 0;

        // Base offense level by statute type (simplified)
        if calculation.offense_statute.contains("2B1") {
            // Fraud offenses
            offense_level = 7;
            if let Some(loss) = calculation.loss_amount {
                offense_level += match loss as i64 {
                    0..=6500 => 0,
                    6501..=15000 => 2,
                    15001..=40000 => 4,
                    40001..=95000 => 6,
                    95001..=150000 => 8,
                    150001..=250000 => 10,
                    250001..=550000 => 12,
                    550001..=1500000 => 14,
                    _ => 16,
                };
            }
        } else if calculation.offense_statute.contains("2D1") {
            // Drug offenses
            offense_level = 20; // Base level varies by drug type/weight
            if let Some(weight) = calculation.drug_weight_grams {
                offense_level += (weight / 1000.0).log2() as i32;
            }
        }

        // Adjustments
        if calculation.weapon_involved {
            offense_level += 2;
        }

        if let Some(injury) = calculation.bodily_injury {
            offense_level += match injury {
                InjuryLevel::None => 0,
                InjuryLevel::MinorBodily => 2,
                InjuryLevel::SeriousBodily => 4,
                InjuryLevel::PermanentLifeThreatening => 6,
                InjuryLevel::Death => 8,
            };
        }

        // Role adjustment
        offense_level += match calculation.defendant_role {
            DefendantRole::MinimalParticipant => -4,
            DefendantRole::MinorParticipant => -2,
            DefendantRole::Average => 0,
            DefendantRole::Manager => 2,
            DefendantRole::Leader => 4,
        };

        // Acceptance of responsibility
        if calculation.acceptance_of_responsibility {
            offense_level -= 2;
            if offense_level >= 16 {
                offense_level -= 1; // Additional reduction for timely plea
            }
        }

        // Obstruction
        if calculation.obstruction {
            offense_level += 2;
        }

        // Calculate range (simplified)
        let min_months = match offense_level {
            i32::MIN..=8 => 0,
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
            _ => 151,
        };

        let max_months = (min_months as f32 * 1.25) as i32;

        let zone = match min_months {
            0..=6 => Zone::A,
            7..=12 => Zone::B,
            13..=18 => Zone::C,
            _ => Zone::D,
        };

        Ok(GuidelinesRange {
            minimum_months: min_months,
            maximum_months: max_months,
            zone,
            mandatory_minimum: None,
            statutory_maximum: None,
        })
    }

    fn get_departure_rates(&self) -> ApiResult<SentencingStatistics> {
        let sentencings = self.get_all_sentencings()?;
        let total = sentencings.len() as i32;

        let with_departures = sentencings.iter()
            .filter(|s| !s.departures.is_empty())
            .count() as i32;

        let upward = sentencings.iter()
            .filter(|s| s.departures.iter()
                .any(|d| matches!(d.direction, DepartureDirection::Upward)))
            .count() as i32;

        let downward = sentencings.iter()
            .filter(|s| s.departures.iter()
                .any(|d| matches!(d.direction, DepartureDirection::Downward)))
            .count() as i32;

        let substantial_assistance = sentencings.iter()
            .filter(|s| s.substantial_assistance.as_ref()
                .map(|sa| sa.departure_granted).unwrap_or(false))
            .count() as i32;

        Ok(SentencingStatistics {
            total_cases: total,
            within_guidelines: total - with_departures,
            upward_departures: upward,
            downward_departures: downward,
            upward_variances: 0,
            downward_variances: 0,
            government_sponsored_below: substantial_assistance,
            substantial_assistance,
            average_sentence_months: 0.0,
            median_sentence_months: 0.0,
            trial_penalty_percentage: 0.0,
        })
    }

    fn get_variance_rates(&self) -> ApiResult<SentencingStatistics> {
        let sentencings = self.get_all_sentencings()?;
        let total = sentencings.len() as i32;

        let with_variance = sentencings.iter()
            .filter(|s| s.variance.is_some())
            .count() as i32;

        let upward = sentencings.iter()
            .filter(|s| s.variance.as_ref()
                .map(|v| matches!(v.direction, VarianceDirection::Above))
                .unwrap_or(false))
            .count() as i32;

        let downward = sentencings.iter()
            .filter(|s| s.variance.as_ref()
                .map(|v| matches!(v.direction, VarianceDirection::Below))
                .unwrap_or(false))
            .count() as i32;

        Ok(SentencingStatistics {
            total_cases: total,
            within_guidelines: total - with_variance,
            upward_departures: 0,
            downward_departures: 0,
            upward_variances: upward,
            downward_variances: downward,
            government_sponsored_below: 0,
            substantial_assistance: 0,
            average_sentence_months: 0.0,
            median_sentence_months: 0.0,
            trial_penalty_percentage: 0.0,
        })
    }

    fn add_departure(&self, sentencing_id: &str, departure: Departure) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        sentencing.departures.push(departure);
        sentencing.updated_at = Utc::now();

        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn add_variance(&self, sentencing_id: &str, variance: Variance) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        sentencing.variance = Some(variance);
        sentencing.updated_at = Utc::now();

        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn get_substantial_assistance_cases(&self) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.substantial_assistance.is_some())
            .collect())
    }

    fn add_special_condition(&self, sentencing_id: &str, condition: SpecialCondition) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        sentencing.special_conditions.push(condition.clone());

        // Also add to supervised release if it exists
        if let Some(ref mut sr) = sentencing.supervised_release {
            sr.special_conditions.push(condition);
        }

        sentencing.updated_at = Utc::now();
        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn update_supervised_release(&self, sentencing_id: &str, release: SupervisedRelease) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        sentencing.supervised_release = Some(release);
        sentencing.updated_at = Utc::now();

        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn find_active_supervision(&self) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.supervised_release.is_some())
            .collect())
    }

    fn add_bop_designation(&self, sentencing_id: &str, designation: BOPDesignation) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        sentencing.bop_designation = Some(designation);
        sentencing.updated_at = Utc::now();

        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn get_rdap_eligible(&self) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        Ok(sentencings.into_iter()
            .filter(|s| s.rdap_eligibility)
            .collect())
    }

    fn get_judge_sentencing_stats(&self, judge_id: &str) -> ApiResult<SentencingStatistics> {
        let sentencings = self.get_all_sentencings()?;
        let judge_sentencings: Vec<_> = sentencings.into_iter()
            .filter(|s| s.judge_id == judge_id)
            .collect();

        let total = judge_sentencings.len() as i32;

        let mut total_months = 0.0;
        let mut sentence_lengths = Vec::new();

        for s in &judge_sentencings {
            if let Some(ref imposed) = s.imposed_sentence {
                let months = imposed.custody_months as f64;
                total_months += months;
                sentence_lengths.push(months);
            }
        }

        sentence_lengths.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if !sentence_lengths.is_empty() {
            sentence_lengths[sentence_lengths.len() / 2]
        } else {
            0.0
        };

        Ok(SentencingStatistics {
            total_cases: total,
            within_guidelines: judge_sentencings.iter()
                .filter(|s| s.departures.is_empty() && s.variance.is_none())
                .count() as i32,
            upward_departures: judge_sentencings.iter()
                .filter(|s| s.departures.iter()
                    .any(|d| matches!(d.direction, DepartureDirection::Upward)))
                .count() as i32,
            downward_departures: judge_sentencings.iter()
                .filter(|s| s.departures.iter()
                    .any(|d| matches!(d.direction, DepartureDirection::Downward)))
                .count() as i32,
            upward_variances: judge_sentencings.iter()
                .filter(|s| s.variance.as_ref()
                    .map(|v| matches!(v.direction, VarianceDirection::Above))
                    .unwrap_or(false))
                .count() as i32,
            downward_variances: judge_sentencings.iter()
                .filter(|s| s.variance.as_ref()
                    .map(|v| matches!(v.direction, VarianceDirection::Below))
                    .unwrap_or(false))
                .count() as i32,
            government_sponsored_below: judge_sentencings.iter()
                .filter(|s| s.substantial_assistance.as_ref()
                    .map(|sa| sa.departure_granted).unwrap_or(false))
                .count() as i32,
            substantial_assistance: judge_sentencings.iter()
                .filter(|s| s.substantial_assistance.is_some())
                .count() as i32,
            average_sentence_months: if total > 0 { total_months / total as f64 } else { 0.0 },
            median_sentence_months: median,
            trial_penalty_percentage: 0.0, // Would need trial vs plea data
        })
    }

    fn get_district_stats(&self) -> ApiResult<SentencingStatistics> {
        let sentencings = self.get_all_sentencings()?;
        let total = sentencings.len() as i32;

        let mut total_months = 0.0;
        for s in &sentencings {
            if let Some(ref imposed) = s.imposed_sentence {
                total_months += imposed.custody_months as f64;
            }
        }

        Ok(SentencingStatistics {
            total_cases: total,
            within_guidelines: sentencings.iter()
                .filter(|s| s.departures.is_empty() && s.variance.is_none())
                .count() as i32,
            upward_departures: sentencings.iter()
                .filter(|s| s.departures.iter()
                    .any(|d| matches!(d.direction, DepartureDirection::Upward)))
                .count() as i32,
            downward_departures: sentencings.iter()
                .filter(|s| s.departures.iter()
                    .any(|d| matches!(d.direction, DepartureDirection::Downward)))
                .count() as i32,
            upward_variances: sentencings.iter()
                .filter(|s| s.variance.as_ref()
                    .map(|v| matches!(v.direction, VarianceDirection::Above))
                    .unwrap_or(false))
                .count() as i32,
            downward_variances: sentencings.iter()
                .filter(|s| s.variance.as_ref()
                    .map(|v| matches!(v.direction, VarianceDirection::Below))
                    .unwrap_or(false))
                .count() as i32,
            government_sponsored_below: sentencings.iter()
                .filter(|s| s.substantial_assistance.as_ref()
                    .map(|sa| sa.departure_granted).unwrap_or(false))
                .count() as i32,
            substantial_assistance: sentencings.iter()
                .filter(|s| s.substantial_assistance.is_some())
                .count() as i32,
            average_sentence_months: if total > 0 { total_months / total as f64 } else { 0.0 },
            median_sentence_months: 0.0,
            trial_penalty_percentage: 0.0,
        })
    }

    fn get_offense_type_stats(&self, offense_type: &str) -> ApiResult<SentencingStatistics> {
        // Filter by offense type in statute
        let sentencings = self.get_all_sentencings()?;
        let filtered: Vec<_> = sentencings.into_iter()
            .filter(|s| {
                // Check if any adjustment mentions the offense type
                s.offense_level.specific_offense_characteristics.iter()
                    .any(|c| c.guideline_section.contains(offense_type))
            })
            .collect();

        let total = filtered.len() as i32;
        Ok(SentencingStatistics {
            total_cases: total,
            within_guidelines: filtered.iter()
                .filter(|s| s.departures.is_empty() && s.variance.is_none())
                .count() as i32,
            upward_departures: 0,
            downward_departures: 0,
            upward_variances: 0,
            downward_variances: 0,
            government_sponsored_below: 0,
            substantial_assistance: 0,
            average_sentence_months: 0.0,
            median_sentence_months: 0.0,
            trial_penalty_percentage: 0.0,
        })
    }

    fn get_trial_penalty_analysis(&self) -> ApiResult<SentencingStatistics> {
        // Would need to compare sentences after trial vs guilty pleas
        // This is a simplified version
        let sentencings = self.get_all_sentencings()?;

        let with_trial: Vec<_> = sentencings.iter()
            .filter(|s| s.offense_level.acceptance_of_responsibility == 0)
            .collect();

        let with_plea: Vec<_> = sentencings.iter()
            .filter(|s| s.offense_level.acceptance_of_responsibility < 0)
            .collect();

        let avg_trial = if !with_trial.is_empty() {
            with_trial.iter()
                .filter_map(|s| s.imposed_sentence.as_ref())
                .map(|s| s.custody_months as f64)
                .sum::<f64>() / with_trial.len() as f64
        } else {
            0.0
        };

        let avg_plea = if !with_plea.is_empty() {
            with_plea.iter()
                .filter_map(|s| s.imposed_sentence.as_ref())
                .map(|s| s.custody_months as f64)
                .sum::<f64>() / with_plea.len() as f64
        } else {
            0.0
        };

        let penalty_percentage = if avg_plea > 0.0 {
            ((avg_trial - avg_plea) / avg_plea) * 100.0
        } else {
            0.0
        };

        Ok(SentencingStatistics {
            total_cases: sentencings.len() as i32,
            within_guidelines: 0,
            upward_departures: 0,
            downward_departures: 0,
            upward_variances: 0,
            downward_variances: 0,
            government_sponsored_below: 0,
            substantial_assistance: 0,
            average_sentence_months: 0.0,
            median_sentence_months: 0.0,
            trial_penalty_percentage: penalty_percentage,
        })
    }

    fn add_prior_sentence(&self, sentencing_id: &str, prior: PriorSentence) -> ApiResult<Sentencing> {
        let mut sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        // Add the prior sentence
        sentencing.criminal_history.prior_sentences.push(prior.clone());

        // Update criminal history points
        sentencing.criminal_history.points += prior.points_assigned;

        // Recalculate category
        sentencing.calculate_criminal_history_category();

        sentencing.updated_at = Utc::now();
        self.save_sentencing(&sentencing)?;
        Ok(sentencing)
    }

    fn calculate_criminal_history_points(&self, sentencing_id: &str) -> ApiResult<i32> {
        let sentencing = self.get_sentencing(sentencing_id)?
            .ok_or_else(|| ApiError::NotFound("Sentencing not found".to_string()))?;

        let total_points = sentencing.criminal_history.points
            + sentencing.criminal_history.status_points
            + sentencing.criminal_history.recency_points;

        Ok(total_points)
    }

    fn find_upcoming_sentencings(&self, days: i32) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        let cutoff = Utc::now() + chrono::Duration::days(days as i64);

        Ok(sentencings.into_iter()
            .filter(|s| {
                if let Some(date) = s.sentencing_date {
                    date <= cutoff && s.imposed_sentence.is_none()
                } else {
                    false
                }
            })
            .collect())
    }

    fn find_appeal_deadline_approaching(&self) -> ApiResult<Vec<Sentencing>> {
        let sentencings = self.get_all_sentencings()?;
        let appeal_deadline = chrono::Duration::days(14); // 14 days to appeal
        let now = Utc::now();

        Ok(sentencings.into_iter()
            .filter(|s| {
                if let Some(date) = s.judgment_date {
                    let deadline = date + appeal_deadline;
                    deadline > now && deadline <= now + chrono::Duration::days(7)
                } else {
                    false
                }
            })
            .collect())
    }
}