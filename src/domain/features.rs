//! Feature flag management for progressive implementation of judicial features
//!
//! This module provides a centralized way to enable/disable features during development
//! and allows for gradual rollout of new functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Feature flags for the judicial system
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct JudicialFeatures {
    /// Core features that are always enabled
    pub core: CoreFeatures,
    /// Advanced features that can be toggled
    pub advanced: AdvancedFeatures,
    /// Experimental features under development
    pub experimental: ExperimentalFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CoreFeatures {
    pub case_management: bool,
    pub basic_docket: bool,
    pub party_management: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdvancedFeatures {
    pub judge_assignment: bool,
    pub automated_scheduling: bool,
    pub sentencing_calculator: bool,
    pub deadline_tracking: bool,
    pub statistical_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExperimentalFeatures {
    pub mdl_proceedings: bool,
    pub ai_assisted_research: bool,
    pub automated_transcription: bool,
    pub predictive_analytics: bool,
}

impl Default for JudicialFeatures {
    fn default() -> Self {
        Self {
            core: CoreFeatures {
                case_management: true,
                basic_docket: true,
                party_management: false,
            },
            advanced: AdvancedFeatures {
                judge_assignment: false,
                automated_scheduling: false,
                sentencing_calculator: false,
                deadline_tracking: false,
                statistical_reporting: false,
            },
            experimental: ExperimentalFeatures {
                mdl_proceedings: false,
                ai_assisted_research: false,
                automated_transcription: false,
                predictive_analytics: false,
            },
        }
    }
}

/// Feature flag manager for runtime feature toggling
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureManager {
    pub features: JudicialFeatures,
    pub overrides: HashMap<String, bool>,
}

impl FeatureManager {
    pub fn new() -> Self {
        Self {
            features: JudicialFeatures::default(),
            overrides: HashMap::new(),
        }
    }

    pub fn with_config(features: JudicialFeatures) -> Self {
        Self {
            features,
            overrides: HashMap::new(),
        }
    }

    pub fn is_enabled(&self, feature_path: &str) -> bool {
        // Check for runtime overrides first
        if let Some(&override_value) = self.overrides.get(feature_path) {
            return override_value;
        }

        // Check the feature configuration
        match feature_path {
            "core.case_management" => self.features.core.case_management,
            "core.basic_docket" => self.features.core.basic_docket,
            "core.party_management" => self.features.core.party_management,
            "advanced.judge_assignment" => self.features.advanced.judge_assignment,
            "advanced.automated_scheduling" => self.features.advanced.automated_scheduling,
            "advanced.sentencing_calculator" => self.features.advanced.sentencing_calculator,
            "advanced.deadline_tracking" => self.features.advanced.deadline_tracking,
            "advanced.statistical_reporting" => self.features.advanced.statistical_reporting,
            "experimental.mdl_proceedings" => self.features.experimental.mdl_proceedings,
            "experimental.ai_assisted_research" => self.features.experimental.ai_assisted_research,
            "experimental.automated_transcription" => self.features.experimental.automated_transcription,
            "experimental.predictive_analytics" => self.features.experimental.predictive_analytics,
            _ => false,
        }
    }

    pub fn set_override(&mut self, feature_path: &str, enabled: bool) {
        self.overrides.insert(feature_path.to_string(), enabled);
    }

    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    pub fn get_enabled_features(&self) -> Vec<String> {
        let mut enabled = Vec::new();

        let features = vec![
            ("core.case_management", self.features.core.case_management),
            ("core.basic_docket", self.features.core.basic_docket),
            ("core.party_management", self.features.core.party_management),
            ("advanced.judge_assignment", self.features.advanced.judge_assignment),
            ("advanced.automated_scheduling", self.features.advanced.automated_scheduling),
            ("advanced.sentencing_calculator", self.features.advanced.sentencing_calculator),
            ("advanced.deadline_tracking", self.features.advanced.deadline_tracking),
            ("advanced.statistical_reporting", self.features.advanced.statistical_reporting),
            ("experimental.mdl_proceedings", self.features.experimental.mdl_proceedings),
            ("experimental.ai_assisted_research", self.features.experimental.ai_assisted_research),
            ("experimental.automated_transcription", self.features.experimental.automated_transcription),
            ("experimental.predictive_analytics", self.features.experimental.predictive_analytics),
        ];

        for (name, is_enabled) in features {
            let final_enabled = self.overrides.get(name).copied().unwrap_or(is_enabled);
            if final_enabled {
                enabled.push(name.to_string());
            }
        }

        enabled
    }
}

/// Feature implementation status for tracking development progress
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FeatureStatus {
    pub name: String,
    pub module: String,
    pub status: ImplementationStatus,
    pub progress_percentage: u8,
    pub estimated_hours: u32,
    pub actual_hours: Option<u32>,
    pub blockers: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub enum ImplementationStatus {
    NotStarted,
    Planning,
    InProgress,
    Testing,
    Completed,
    Blocked,
    Deprecated,
}

/// Track overall project implementation progress
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ImplementationTracker {
    pub features: Vec<FeatureStatus>,
}

impl ImplementationTracker {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    pub fn add_feature(&mut self, feature: FeatureStatus) {
        self.features.push(feature);
    }

    pub fn update_status(&mut self, name: &str, status: ImplementationStatus) {
        if let Some(feature) = self.features.iter_mut().find(|f| f.name == name) {
            feature.status = status;
        }
    }

    pub fn update_progress(&mut self, name: &str, progress: u8) {
        if let Some(feature) = self.features.iter_mut().find(|f| f.name == name) {
            feature.progress_percentage = progress.min(100);
        }
    }

    pub fn get_summary(&self) -> ImplementationSummary {
        let total = self.features.len();
        let completed = self.features.iter().filter(|f| f.status == ImplementationStatus::Completed).count();
        let in_progress = self.features.iter().filter(|f| f.status == ImplementationStatus::InProgress).count();
        let blocked = self.features.iter().filter(|f| f.status == ImplementationStatus::Blocked).count();

        let total_estimated: u32 = self.features.iter().map(|f| f.estimated_hours).sum();
        let total_actual: u32 = self.features.iter().filter_map(|f| f.actual_hours).sum();

        let overall_progress: f32 = if total > 0 {
            self.features.iter().map(|f| f.progress_percentage as f32).sum::<f32>() / (total as f32)
        } else {
            0.0
        };

        ImplementationSummary {
            total_features: total,
            completed,
            in_progress,
            blocked,
            not_started: total - completed - in_progress - blocked,
            overall_progress_percentage: overall_progress as u8,
            total_estimated_hours: total_estimated,
            total_actual_hours: total_actual,
        }
    }

    pub fn get_blocked_features(&self) -> Vec<&FeatureStatus> {
        self.features.iter().filter(|f| f.status == ImplementationStatus::Blocked).collect()
    }

    pub fn get_ready_to_start(&self) -> Vec<&FeatureStatus> {
        self.features.iter().filter(|f| {
            f.status == ImplementationStatus::NotStarted &&
            f.dependencies.iter().all(|dep| {
                self.features.iter().any(|other|
                    other.name == *dep && other.status == ImplementationStatus::Completed
                )
            })
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImplementationSummary {
    pub total_features: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub not_started: usize,
    pub overall_progress_percentage: u8,
    pub total_estimated_hours: u32,
    pub total_actual_hours: u32,
}