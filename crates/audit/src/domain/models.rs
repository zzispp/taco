use std::collections::BTreeMap;

use time::OffsetDateTime;

use super::{AuditStatus, BusinessType, LoginEventType, OperatorType};

pub const LOCATION_KIND_RESOLVED: &str = "resolved";
pub const LOCATION_KIND_INTERNAL: &str = "internal";
pub const LOCATION_KIND_UNKNOWN: &str = "unknown";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AuditLocation {
    Resolved(String),
    Internal,
    #[default]
    Unknown,
}

impl AuditLocation {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Resolved(_) => LOCATION_KIND_RESOLVED,
            Self::Internal => LOCATION_KIND_INTERNAL,
            Self::Unknown => LOCATION_KIND_UNKNOWN,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Self::Resolved(value) => value,
            Self::Internal | Self::Unknown => "",
        }
    }

    pub fn from_persisted(kind: &str, text: String) -> Option<Self> {
        match (kind, text.is_empty()) {
            (LOCATION_KIND_RESOLVED, false) => Some(Self::Resolved(text)),
            (LOCATION_KIND_INTERNAL, true) => Some(Self::Internal),
            (LOCATION_KIND_UNKNOWN, true) => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationLogSummary {
    pub id: String,
    pub title_key: String,
    pub business_type: BusinessType,
    pub handler: String,
    pub request_method: String,
    pub operator_type: OperatorType,
    pub operator_name: String,
    pub department_name: String,
    pub operation_url: String,
    pub operation_ip: String,
    pub operation_location: AuditLocation,
    pub status: AuditStatus,
    pub operation_time: OffsetDateTime,
    pub cost_time_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationLogDetail {
    pub summary: OperationLogSummary,
    pub request_id: String,
    pub operator_id: Option<String>,
    pub department_id: Option<String>,
    pub request_params: String,
    pub response_result: String,
    pub error_message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewOperationLog {
    pub detail: OperationLogDetail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginLog {
    pub id: String,
    pub user_id: Option<String>,
    pub username: String,
    pub ip_address: String,
    pub login_location: AuditLocation,
    pub browser: String,
    pub os: String,
    pub status: AuditStatus,
    pub event_type: LoginEventType,
    pub message_key: String,
    pub message_params: BTreeMap<String, String>,
    pub login_time: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewLoginLog {
    pub request_id: String,
    pub route: String,
    pub user_id: Option<String>,
    pub username: String,
    pub ip_address: String,
    pub login_location: AuditLocation,
    pub browser: String,
    pub os: String,
    pub status: AuditStatus,
    pub event_type: LoginEventType,
    pub message_key: String,
    pub message_params: BTreeMap<String, String>,
    pub login_time: OffsetDateTime,
}
