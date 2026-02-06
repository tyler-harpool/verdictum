//! CVRA Victim notification domain model
//!
//! Implements Crime Victims' Rights Act (18 U.S.C. § 3771) notification
//! requirements for federal criminal cases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;

/// Type of victim
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VictimType {
    Individual,
    Business,
    Organization,
    Government,
    Minor,
    Other(String),
}

/// Method of notification delivery
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationMethod {
    Email,
    Mail,
    Phone,
    InPerson,
    VictimAdvocate,
}

/// Type of notification sent to victim
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    CaseFiled,
    HearingScheduled,
    PleaAgreement,
    Sentencing,
    Release,
    Escape,
    Restitution,
    AppealFiled,
    CaseDisposition,
    Other(String),
}

/// Victim's notification preferences
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationPreferences {
    pub preferred_method: NotificationMethod,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mailing_address: Option<String>,
    pub victim_advocate: Option<String>,
    pub opt_out: bool,
}

/// A notification sent to a victim
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VictimNotification {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub sent_at: DateTime<Utc>,
    pub method: NotificationMethod,
    pub content_summary: String,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// A victim associated with a criminal case per CVRA
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Victim {
    pub id: Uuid,
    pub case_id: Uuid,
    pub name: String,
    pub victim_type: VictimType,
    pub notification_preferences: NotificationPreferences,
    pub notifications: Vec<VictimNotification>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new victim record
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateVictimRequest {
    pub name: String,
    #[serde(rename = "victimType", default = "default_victim_type")]
    pub victim_type: VictimType,
    #[serde(rename = "preferredMethod", default = "default_notification_method")]
    pub preferred_method: NotificationMethod,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(rename = "mailingAddress")]
    pub mailing_address: Option<String>,
    #[serde(rename = "victimAdvocate")]
    pub victim_advocate: Option<String>,
}

fn default_victim_type() -> VictimType {
    VictimType::Individual
}

fn default_notification_method() -> NotificationMethod {
    NotificationMethod::Email
}

/// Request to send a notification to a victim
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SendNotificationRequest {
    #[serde(rename = "notificationType")]
    pub notification_type: NotificationType,
    #[serde(rename = "contentSummary")]
    pub content_summary: String,
}
