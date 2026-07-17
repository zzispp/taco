#[macro_export]
macro_rules! info_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        $crate::__tracing::info!(target: module_path!(), __taco_system_log = true, message = %$message, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
}

#[macro_export]
macro_rules! warn_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        $crate::__tracing::warn!(target: module_path!(), __taco_system_log = true, message = %$message, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        $crate::__tracing::error!(target: module_path!(), __taco_system_log = true, message = %$message, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
    ($message:expr, $error:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let __taco_safe_error = $crate::safe_error_value(&$error);
        $crate::__tracing::error!(target: module_path!(), __taco_system_log = true, message = %$message, error = %__taco_safe_error, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
}

#[macro_export]
macro_rules! debug_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        $crate::__tracing::debug!(target: module_path!(), __taco_system_log = true, message = %$message, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
}

#[macro_export]
macro_rules! trace_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        $crate::__tracing::trace!(target: module_path!(), __taco_system_log = true, message = %$message, $($field = %$crate::safe_field_value(stringify!($field), &$value)),*);
    }};
}
