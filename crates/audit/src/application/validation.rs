use std::collections::HashSet;

use super::{AuditError, AuditResult, localized};

pub fn validate_ids(ids: Vec<String>) -> AuditResult<Vec<String>> {
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

fn invalid_ids() -> AuditError {
    AuditError::InvalidInput(localized("errors.audit.ids_required"))
}

#[cfg(test)]
mod tests {
    use super::validate_ids;

    #[test]
    fn batch_ids_must_be_nonempty_trimmed_and_unique() {
        assert_eq!(validate_ids(vec!["a".into(), "b".into()]).unwrap(), ["a", "b"]);
        for ids in [Vec::new(), vec![" ".into()], vec!["a".into(), "a".into()]] {
            assert!(validate_ids(ids).is_err());
        }
    }
}
