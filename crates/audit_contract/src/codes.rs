use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStream {
    Operation,
    Security,
}

impl AuditStream {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Operation => "operation",
            Self::Security => "security",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "operation" => Some(Self::Operation),
            "security" => Some(Self::Security),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessType {
    Other,
    Insert,
    Update,
    Delete,
    Grant,
    Export,
    Import,
    Force,
    GenerateCode,
    Clean,
}

impl BusinessType {
    const ALL: [Self; 10] = [
        Self::Other,
        Self::Insert,
        Self::Update,
        Self::Delete,
        Self::Grant,
        Self::Export,
        Self::Import,
        Self::Force,
        Self::GenerateCode,
        Self::Clean,
    ];

    pub const fn code(self) -> i16 {
        match self {
            Self::Other => 0,
            Self::Insert => 1,
            Self::Update => 2,
            Self::Delete => 3,
            Self::Grant => 4,
            Self::Export => 5,
            Self::Import => 6,
            Self::Force => 7,
            Self::GenerateCode => 8,
            Self::Clean => 9,
        }
    }

    pub fn parse(code: i16) -> Option<Self> {
        Self::ALL.into_iter().find(|value| value.code() == code)
    }

    pub const fn message_key(self) -> &'static str {
        match self {
            Self::Other => "audit.business_type.other",
            Self::Insert => "audit.business_type.insert",
            Self::Update => "audit.business_type.update",
            Self::Delete => "audit.business_type.delete",
            Self::Grant => "audit.business_type.grant",
            Self::Export => "audit.business_type.export",
            Self::Import => "audit.business_type.import",
            Self::Force => "audit.business_type.force",
            Self::GenerateCode => "audit.business_type.generate_code",
            Self::Clean => "audit.business_type.clean",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Success,
    Failure,
}

impl AuditStatus {
    pub const fn code(self) -> i16 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
        }
    }

    pub const fn parse(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Success),
            1 => Some(Self::Failure),
            _ => None,
        }
    }

    pub const fn message_key(self) -> &'static str {
        match self {
            Self::Success => "audit.status.success",
            Self::Failure => "audit.status.failure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatorType {
    Other,
    Manage,
    Mobile,
}

impl OperatorType {
    pub const fn code(self) -> i16 {
        match self {
            Self::Other => 0,
            Self::Manage => 1,
            Self::Mobile => 2,
        }
    }

    pub const fn parse(code: i16) -> Option<Self> {
        match code {
            0 => Some(Self::Other),
            1 => Some(Self::Manage),
            2 => Some(Self::Mobile),
            _ => None,
        }
    }

    pub const fn message_key(self) -> &'static str {
        match self {
            Self::Other => "audit.operator_type.other",
            Self::Manage => "audit.operator_type.manage",
            Self::Mobile => "audit.operator_type.mobile",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginEventType {
    LoginSuccess,
    LoginFailure,
    RegisterSuccess,
    RegisterFailure,
    LogoutSuccess,
    LogoutFailure,
    RefreshSuccess,
    RefreshFailure,
}

impl LoginEventType {
    pub const ALL: [Self; 8] = [
        Self::LoginSuccess,
        Self::LoginFailure,
        Self::RegisterSuccess,
        Self::RegisterFailure,
        Self::LogoutSuccess,
        Self::LogoutFailure,
        Self::RefreshSuccess,
        Self::RefreshFailure,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::LoginSuccess => "login_success",
            Self::LoginFailure => "login_failure",
            Self::RegisterSuccess => "register_success",
            Self::RegisterFailure => "register_failure",
            Self::LogoutSuccess => "logout_success",
            Self::LogoutFailure => "logout_failure",
            Self::RefreshSuccess => "refresh_success",
            Self::RefreshFailure => "refresh_failure",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "login_success" => Some(Self::LoginSuccess),
            "login_failure" => Some(Self::LoginFailure),
            "register_success" => Some(Self::RegisterSuccess),
            "register_failure" => Some(Self::RegisterFailure),
            "logout_success" => Some(Self::LogoutSuccess),
            "logout_failure" => Some(Self::LogoutFailure),
            "refresh_success" => Some(Self::RefreshSuccess),
            "refresh_failure" => Some(Self::RefreshFailure),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_codes_are_explicit_and_round_trip() {
        assert_eq!(AuditStream::parse(AuditStream::Operation.code()), Some(AuditStream::Operation));
        assert_eq!(BusinessType::parse(BusinessType::Clean.code()), Some(BusinessType::Clean));
        assert_eq!(AuditStatus::parse(AuditStatus::Failure.code()), Some(AuditStatus::Failure));
        assert_eq!(OperatorType::parse(OperatorType::Manage.code()), Some(OperatorType::Manage));
        assert_eq!(
            LoginEventType::ALL.map(LoginEventType::code),
            [
                "login_success",
                "login_failure",
                "register_success",
                "register_failure",
                "logout_success",
                "logout_failure",
                "refresh_success",
                "refresh_failure",
            ]
        );
        for event_type in LoginEventType::ALL {
            assert_eq!(LoginEventType::parse(event_type.code()), Some(event_type));
        }
    }
}
