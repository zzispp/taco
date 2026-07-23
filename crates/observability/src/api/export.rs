use types::http::{Locale, translate_message};

use crate::application::SystemLogExportLayout;

const HEADER_KEYS: [&str; 6] = [
    "excel.observability.system.headers.id",
    "excel.observability.system.headers.time",
    "excel.observability.system.headers.level",
    "excel.observability.system.headers.target",
    "excel.observability.system.headers.message",
    "excel.observability.system.headers.fields",
];
const CONTINUATION_HEADER_KEYS: [&str; 4] = [
    "excel.observability.system.continuation_headers.log_id",
    "excel.observability.system.continuation_headers.value_kind",
    "excel.observability.system.continuation_headers.part",
    "excel.observability.system.continuation_headers.content",
];

pub(super) fn system_log_export_layout(locale: Locale) -> SystemLogExportLayout {
    SystemLogExportLayout::new(
        translate_message(locale, "excel.observability.system.sheet"),
        HEADER_KEYS.map(|key| translate_message(locale, key)),
        CONTINUATION_HEADER_KEYS.map(|key| translate_message(locale, key)),
    )
}

#[cfg(test)]
mod tests {
    use types::http::Locale;

    use crate::application::SystemLogExportLayout;

    use super::system_log_export_layout;

    #[test]
    fn export_layout_contains_all_localized_columns() {
        let layout = system_log_export_layout(Locale::En);

        assert_eq!(
            layout,
            SystemLogExportLayout::new(
                "System logs".into(),
                ["Log ID", "Occurred at", "Level", "Target", "Message", "Structured fields"].map(str::to_owned),
                ["Log ID", "Value kind", "Part", "Content"].map(str::to_owned),
            )
        );
    }
}
