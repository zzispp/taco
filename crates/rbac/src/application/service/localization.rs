use kernel::error::LocalizedError;

pub(super) fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub(super) fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}
