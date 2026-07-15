use kernel::excel::{StreamingXlsxWriter, TemporaryXlsxFile};
use types::{
    http::{Locale, translate_message},
    rbac::Role,
};

use crate::application::{RbacError, RbacResult, RoleExportSink};

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

pub struct RoleXlsxExport {
    writer: StreamingXlsxWriter,
}

impl RoleXlsxExport {
    pub fn new(locale: Locale) -> RbacResult<Self> {
        let writer = StreamingXlsxWriter::new(&text(locale, ROLE_SHEET_KEY), &localized_headers(locale)).map_err(RbacError::Infrastructure)?;
        Ok(Self { writer })
    }

    pub fn finish(self) -> RbacResult<TemporaryXlsxFile> {
        self.writer.finish().map_err(RbacError::Infrastructure)
    }
}

impl RoleExportSink for RoleXlsxExport {
    fn append(&mut self, roles: &[Role]) -> RbacResult<()> {
        let rows = roles.iter().map(role_row).collect::<Vec<_>>();
        self.writer.append_rows(&rows).map_err(RbacError::Infrastructure)
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
    use types::http::Locale;

    use super::RoleXlsxExport;

    #[test]
    fn export_roles_headers_use_requested_locale() {
        let artifact = RoleXlsxExport::new(Locale::En).unwrap().finish().unwrap();
        let bytes = std::fs::read(artifact.path()).unwrap();
        let rows = kernel::excel::read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0][0], "Role ID");
        assert_eq!(rows[0][1], "Role name");
    }
}
