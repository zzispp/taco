use std::collections::HashSet;

use serde_json::Value;

use super::{SchedulerError, SchedulerResult};

pub fn require_text(value: &str, key: &'static str) -> SchedulerResult<()> {
    if value.trim().is_empty() {
        return Err(SchedulerError::InvalidInput(super::error::localized(key)));
    }
    Ok(())
}

pub fn validate_json_object(value: &Value) -> SchedulerResult<()> {
    if value.is_object() {
        return Ok(());
    }
    Err(SchedulerError::InvalidInput(super::error::localized("errors.scheduler.params_must_be_object")))
}

pub fn validate_ids(ids: Vec<String>) -> SchedulerResult<Vec<String>> {
    if ids.is_empty() {
        return Err(invalid_ids());
    }
    let mut unique = HashSet::with_capacity(ids.len());
    for id in &ids {
        if id.trim().is_empty() || id.trim() != id || !unique.insert(id.as_str()) {
            return Err(invalid_ids());
        }
    }
    Ok(ids)
}

fn invalid_ids() -> SchedulerError {
    SchedulerError::InvalidInput(super::error::localized("errors.scheduler.ids_required"))
}

#[cfg(test)]
mod tests {
    use super::validate_ids;
    use crate::application::SchedulerError;

    #[test]
    fn valid_ids_preserve_request_order() {
        let ids = vec!["job-2".to_owned(), "job-1".to_owned()];
        assert_eq!(validate_ids(ids.clone()).expect("valid ids must pass"), ids);
    }

    #[test]
    fn empty_and_blank_ids_fail_explicitly() {
        assert_invalid_ids(validate_ids(Vec::new()).expect_err("an empty request must fail"));
        assert_invalid_ids(validate_ids(vec!["   ".into()]).expect_err("a blank id must fail"));
    }

    #[test]
    fn duplicate_ids_fail_explicitly() {
        let error = validate_ids(vec!["job-1".into(), "job-1".into()]).expect_err("duplicate ids must fail");
        assert_invalid_ids(error);
    }

    #[test]
    fn mixed_valid_and_blank_ids_fail_as_one_request() {
        let error = validate_ids(vec!["job-1".into(), " ".into()]).expect_err("mixed invalid ids must fail");
        assert_invalid_ids(error);
    }

    fn assert_invalid_ids(error: SchedulerError) {
        match error {
            SchedulerError::InvalidInput(details) => assert_eq!(details.key(), "errors.scheduler.ids_required"),
            other => panic!("expected invalid input, got {other:?}"),
        }
    }
}
