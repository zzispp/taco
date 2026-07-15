pub use audit_contract::{AuditStatus, BusinessType, LoginEventType, OperatorType};

#[cfg(test)]
mod tests {
    use super::{AuditStatus, BusinessType, LoginEventType, OperatorType};

    #[test]
    fn persistent_codes_are_explicit_and_round_trip() {
        for code in 0..=9 {
            let value = BusinessType::parse(code).expect("known business type");
            assert_eq!(value.code(), code);
        }
        for code in 0..=1 {
            assert_eq!(AuditStatus::parse(code).unwrap().code(), code);
        }
        for code in 0..=2 {
            assert_eq!(OperatorType::parse(code).unwrap().code(), code);
        }
        assert_eq!(LoginEventType::parse("refresh_failure").unwrap().code(), "refresh_failure");
    }

    #[test]
    fn unknown_persistent_codes_are_rejected() {
        assert_eq!(BusinessType::parse(10), None);
        assert_eq!(AuditStatus::parse(2), None);
        assert_eq!(OperatorType::parse(3), None);
        assert_eq!(LoginEventType::parse("captcha_redeem_success"), None);
    }
}
