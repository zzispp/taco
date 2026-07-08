use kernel::excel::write_xlsx;
use kernel::pagination::PageRequest;
use types::{
    http::{Locale, translate_message},
    rbac::Role,
};

use crate::application::{RbacError, RbacResult, RoleListFilter};

use super::handlers::RoleExportQuery;

const ROLE_SHEET_KEY: &str = "excel.rbac.role.sheet";
const ROLE_HEADER_KEYS: &[&str] = &[
    "excel.rbac.role.headers.role_id",
    "excel.rbac.role.headers.role_name",
    "excel.rbac.role.headers.role_key",
    "excel.rbac.role.headers.role_sort",
    "excel.rbac.role.headers.data_scope",
    "excel.rbac.role.headers.status",
    "excel.rbac.role.headers.remark",
    "excel.rbac.role.headers.create_time",
];

pub fn export_roles_xlsx(roles: &[Role], locale: Locale) -> RbacResult<Vec<u8>> {
    let rows = roles.iter().map(role_row).collect::<Vec<_>>();
    write_xlsx(&text(locale, ROLE_SHEET_KEY), &localized_headers(locale), &rows).map_err(RbacError::Infrastructure)
}

pub fn role_export_page(query: &RoleExportQuery, page: u64, page_size: u64) -> RoleListFilter {
    RoleListFilter {
        page: PageRequest { page, page_size },
        role_name: query.role_name.clone(),
        role_key: query.role_key.clone(),
        status: query.status.clone(),
        system: query.system,
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
}

fn localized_headers(locale: Locale) -> Vec<String> {
    ROLE_HEADER_KEYS.iter().map(|key| text(locale, key)).collect()
}

fn text(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

fn role_row(role: &Role) -> Vec<String> {
    vec![
        role.role_id.clone(),
        role.role_name.clone(),
        role.role_key.clone(),
        role.role_sort.to_string(),
        role.data_scope.clone(),
        role.status.clone(),
        role.remark.clone().unwrap_or_default(),
        role.create_time.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::export_roles_xlsx;
    use types::http::Locale;

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn export_roles_headers_use_requested_locale() {
        let rows = kernel::excel::read_xlsx(&export_roles_xlsx(&[], Locale::En).unwrap()).unwrap();

        assert_eq!(rows[0][0], "Role ID");
        assert_eq!(rows[0][1], "Role name");
    }
}
