use kernel::excel::write_xlsx;
use kernel::pagination::PageRequest;
use types::rbac::Role;

use crate::application::{RbacError, RbacResult, RoleListFilter};

use super::handlers::RoleExportQuery;

const ROLE_HEADERS: &[&str] = &["角色序号", "角色名称", "权限字符", "显示顺序", "数据范围", "角色状态", "备注", "创建时间"];

pub fn export_roles_xlsx(roles: &[Role]) -> RbacResult<Vec<u8>> {
    let rows = roles.iter().map(role_row).collect::<Vec<_>>();
    write_xlsx("角色数据", ROLE_HEADERS, &rows).map_err(|error| RbacError::Infrastructure(error))
}

pub fn role_export_page(query: &RoleExportQuery, page: u64, page_size: u64) -> RoleListFilter {
    RoleListFilter {
        page: PageRequest { page, page_size },
        role_name: query.role_name.clone(),
        role_key: query.role_key.clone(),
        status: query.status.clone(),
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
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
