#[macro_export]
macro_rules! info_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::info_with_fields_impl($message, fields);
    }};
}

#[macro_export]
macro_rules! warn_with_fields {
    ($message:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::warn_with_fields_impl($message, fields);
    }};
}

#[macro_export]
macro_rules! error_with_fields {
    ($message:expr, $error:expr, $($field:ident = $value:expr),* $(,)?) => {{
        let fields: &[(&str, &dyn std::fmt::Display)] = &[
            $((stringify!($field), &$value),)*
        ];
        $crate::error_with_fields_impl($message, $error, fields);
    }};
}
