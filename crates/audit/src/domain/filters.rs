use time::OffsetDateTime;

use super::{AuditStatus, BusinessType, LoginEventType};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    #[default]
    Desc,
}

impl SortDirection {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }

    pub const fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OperationSortField {
    #[default]
    OperationTime,
    BusinessType,
    Status,
    OperatorName,
    CostTime,
}

impl OperationSortField {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "oper_time" => Some(Self::OperationTime),
            "business_type" => Some(Self::BusinessType),
            "status" => Some(Self::Status),
            "oper_name" => Some(Self::OperatorName),
            "cost_time" => Some(Self::CostTime),
            _ => None,
        }
    }

    pub const fn column(self) -> &'static str {
        match self {
            Self::OperationTime => "oper_time",
            Self::BusinessType => "business_type",
            Self::Status => "status",
            Self::OperatorName => "oper_name",
            Self::CostTime => "cost_time",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::OperationTime => "oper_time",
            Self::BusinessType => "business_type",
            Self::Status => "status",
            Self::OperatorName => "oper_name",
            Self::CostTime => "cost_time",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoginSortField {
    #[default]
    LoginTime,
    Username,
    IpAddress,
    Status,
}

impl LoginSortField {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "login_time" => Some(Self::LoginTime),
            "user_name" => Some(Self::Username),
            "ipaddr" => Some(Self::IpAddress),
            "status" => Some(Self::Status),
            _ => None,
        }
    }

    pub const fn column(self) -> &'static str {
        match self {
            Self::LoginTime => "login_time",
            Self::Username => "user_name",
            Self::IpAddress => "ipaddr",
            Self::Status => "status",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::LoginTime => "login_time",
            Self::Username => "user_name",
            Self::IpAddress => "ipaddr",
            Self::Status => "status",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OperationLogFilter {
    pub title: Option<String>,
    pub title_keys: Vec<String>,
    pub business_types: Vec<BusinessType>,
    pub status: Option<AuditStatus>,
    pub operator_name: Option<String>,
    pub operation_ip: Option<String>,
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
    pub sort_field: OperationSortField,
    pub sort_direction: SortDirection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoginLogFilter {
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub status: Option<AuditStatus>,
    pub event_type: Option<LoginEventType>,
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
    pub sort_field: LoginSortField,
    pub sort_direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::{LoginSortField, OperationSortField, SortDirection};

    #[test]
    fn sort_fields_are_an_explicit_wire_whitelist() {
        assert_eq!(OperationSortField::parse("oper_time"), Some(OperationSortField::OperationTime));
        assert_eq!(OperationSortField::parse("title"), None);
        assert_eq!(LoginSortField::parse("ipaddr"), Some(LoginSortField::IpAddress));
        for invalid in ["operation_time", "username", "1 desc", "unknown"] {
            assert_eq!(OperationSortField::parse(invalid), None);
            assert_eq!(LoginSortField::parse(invalid), None);
        }
        assert_eq!(SortDirection::parse("DESC"), None);
    }
}
