use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataScope {
    All,
    Custom,
    Department,
    DepartmentAndChildren,
    SelfOnly,
}

impl DataScope {
    pub const fn code(self) -> &'static str {
        match self {
            Self::All => "1",
            Self::Custom => "2",
            Self::Department => "3",
            Self::DepartmentAndChildren => "4",
            Self::SelfOnly => "5",
        }
    }
}

impl TryFrom<&str> for DataScope {
    type Error = InvalidDataScope;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1" => Ok(Self::All),
            "2" => Ok(Self::Custom),
            "3" => Ok(Self::Department),
            "4" => Ok(Self::DepartmentAndChildren),
            "5" => Ok(Self::SelfOnly),
            _ => Err(InvalidDataScope),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataScopeFilter {
    pub data_scope: DataScope,
    pub user_id: String,
    pub dept_id: Option<String>,
    pub dept_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("invalid data scope")]
pub struct InvalidDataScope;

#[cfg(test)]
mod tests {
    use super::DataScope;

    #[test]
    fn data_scope_codes_round_trip_and_order_by_permission_width() {
        let scopes = [
            DataScope::All,
            DataScope::Custom,
            DataScope::Department,
            DataScope::DepartmentAndChildren,
            DataScope::SelfOnly,
        ];

        for scope in scopes {
            assert_eq!(DataScope::try_from(scope.code()), Ok(scope));
        }
        assert_eq!(scopes.into_iter().min(), Some(DataScope::All));
        assert!(DataScope::try_from("unknown").is_err());
    }
}
