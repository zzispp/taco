use std::collections::BTreeSet;

use types::http::{Locale, translate_message};

mod support;
use support::{parsed_catalogs, parsed_responsibility_catalogs, placeholders};

const FILE_ERROR_KEYS: &[&str] = &[
    "errors.file.active_business_references",
    "errors.file.active_upload_target",
    "errors.file.available_capacity_exceeds_total",
    "errors.file.avatar_reference",
    "errors.file.capacity_exceeded",
    "errors.file.cleanup_batch_size_invalid",
    "errors.file.cleanup_batch_size_too_large",
    "errors.file.config_default_quota_invalid",
    "errors.file.config_invalid_json",
    "errors.file.config_max_file_bytes_invalid",
    "errors.file.config_upload_part_bytes_invalid",
    "errors.file.config_upload_session_inactivity_invalid",
    "errors.file.content_type_invalid",
    "errors.file.cursor_limit_invalid",
    "errors.file.cursor_limit_too_large",
    "errors.file.cursor_malformed",
    "errors.file.cursor_query_mismatch",
    "errors.file.declared_digest_required",
    "errors.file.digest_format_invalid",
    "errors.file.digest_length_invalid",
    "errors.file.digest_mismatch",
    "errors.file.empty_file",
    "errors.file.entry_name_forbidden_path",
    "errors.file.entry_name_invalid",
    "errors.file.entry_type_invalid",
    "errors.file.file_ids_required",
    "errors.file.file_ids_too_many",
    "errors.file.file_size_exceeded",
    "errors.file.folder_download_forbidden",
    "errors.file.idempotency_key_invalid_utf8",
    "errors.file.idempotency_key_invalid",
    "errors.file.idempotency_key_required",
    "errors.file.idempotency_key_too_long",
    "errors.file.identifier_invalid",
    "errors.file.image_source_too_large",
    "errors.file.infrastructure",
    "errors.file.invalid_input",
    "errors.file.invalid_part",
    "errors.file.invalid_upload_transition",
    "errors.file.name_conflict",
    "errors.file.not_found",
    "errors.file.object_key_forbidden_path",
    "errors.file.object_key_invalid",
    "errors.file.parent_folder_invalid",
    "errors.file.part_digest_header_required",
    "errors.file.provider_cleanup_payload_invalid",
    "errors.file.provider_io",
    "errors.file.provider_key_invalid",
    "errors.file.provider_object_mismatch",
    "errors.file.provider_part_ref_invalid",
    "errors.file.provider_unavailable",
    "errors.file.provider_upload_ref_invalid",
    "errors.file.purge_requires_trashed",
    "errors.file.quota_exceeded",
    "errors.file.quota_release_exceeds_usage",
    "errors.file.quota_too_large",
    "errors.file.range_header_invalid",
    "errors.file.range_not_satisfiable",
    "errors.file.response_header_invalid",
    "errors.file.retention_days_too_large",
    "errors.file.size_mismatch",
    "errors.file.sort_field_invalid",
    "errors.file.sort_order_invalid",
    "errors.file.space_identifier_required",
    "errors.file.system_folder_immutable",
    "errors.file.tag_invalid",
    "errors.file.tag_required",
    "errors.file.tag_too_long",
    "errors.file.time_filter_invalid",
    "errors.file.upload_incomplete",
    "errors.file.upload_completion_in_progress",
    "errors.file.upload_intent_terminal",
    "errors.file.upload_not_found",
    "errors.file.upload_part_conflict",
    "errors.file.upload_part_size_invalid",
    "errors.file.upload_result_unavailable",
];

#[test]
fn file_catalog_contract_is_complete() {
    let expected = FILE_ERROR_KEYS.iter().map(|key| (*key).to_owned()).collect::<BTreeSet<_>>();

    for (locale, catalog) in parsed_catalogs() {
        for key in FILE_ERROR_KEYS {
            assert!(catalog.contains_key(*key), "missing file locale key {locale}:{key}");
            assert!(
                placeholders(&catalog[*key]).is_empty(),
                "unexpected placeholder in file locale key {locale}:{key}"
            );
        }
    }

    for (locale, catalog) in parsed_responsibility_catalogs("file") {
        let actual = catalog.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(actual, expected, "unexpected file locale keys for {locale}");
    }
}

#[test]
fn file_validation_messages_translate_in_all_locales() {
    assert_eq!(translate_message(Locale::ZhCn, "errors.file.entry_name_invalid"), "文件名为空或过长");
    assert_eq!(
        translate_message(Locale::En, "errors.file.entry_name_invalid"),
        "The file name is blank or too long"
    );
    assert_eq!(translate_message(Locale::ZhTw, "errors.file.entry_name_invalid"), "檔案名稱為空白或過長");
}
