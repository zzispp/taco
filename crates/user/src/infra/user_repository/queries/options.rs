use sqlx::query_as;
use storage::{StorageError, StorageResult};
use types::{
    rbac::RoleOption,
    system::{Post, TreeSelectNode},
    user::UserFormOptions,
};

use super::UserQueries;

impl UserQueries {
    pub async fn form_options(&self) -> StorageResult<UserFormOptions> {
        Ok(UserFormOptions {
            roles: self.role_options().await?,
            posts: self.post_options().await?,
            depts: dept_tree(self.dept_options().await?),
        })
    }

    async fn role_options(&self) -> StorageResult<Vec<RoleOption>> {
        query_as::<_, RoleOptionRecord>("SELECT role_id,role_name,role_key,status FROM sys_role WHERE del_flag='0' ORDER BY role_sort ASC")
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(role_option).collect())
            .map_err(StorageError::from)
    }

    async fn post_options(&self) -> StorageResult<Vec<Post>> {
        query_as::<_, PostOptionRecord>("SELECT post_id,post_code,post_name,post_sort,status,remark FROM sys_post ORDER BY post_sort ASC")
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(post_option).collect())
            .map_err(StorageError::from)
    }

    async fn dept_options(&self) -> StorageResult<Vec<DeptOptionRecord>> {
        query_as::<_, DeptOptionRecord>("SELECT dept_id,parent_id,dept_name,status FROM sys_dept WHERE del_flag='0' ORDER BY parent_id ASC,order_num ASC")
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}

#[derive(sqlx::FromRow)]
struct RoleOptionRecord {
    role_id: String,
    role_name: String,
    role_key: String,
    status: String,
}

#[derive(sqlx::FromRow)]
struct PostOptionRecord {
    post_id: String,
    post_code: String,
    post_name: String,
    post_sort: i64,
    status: String,
    remark: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct DeptOptionRecord {
    dept_id: String,
    parent_id: String,
    dept_name: String,
    status: String,
}

fn role_option(record: RoleOptionRecord) -> RoleOption {
    RoleOption {
        role_id: record.role_id,
        role_name: record.role_name,
        role_key: record.role_key,
        status: record.status,
    }
}

fn post_option(record: PostOptionRecord) -> Post {
    Post {
        post_id: record.post_id,
        post_code: record.post_code,
        post_name: record.post_name,
        post_sort: record.post_sort,
        status: record.status,
        remark: record.remark,
        create_time: String::new(),
    }
}

fn dept_tree(records: Vec<DeptOptionRecord>) -> Vec<TreeSelectNode> {
    records
        .iter()
        .filter(|record| is_root(record, &records))
        .map(|record| dept_node(record, &records))
        .collect()
}

fn dept_node(record: &DeptOptionRecord, records: &[DeptOptionRecord]) -> TreeSelectNode {
    TreeSelectNode {
        id: record.dept_id.clone(),
        label: record.dept_name.clone(),
        parent_id: record.parent_id.clone(),
        disabled: record.status != constants::system::STATUS_NORMAL,
        children: records
            .iter()
            .filter(|child| child.parent_id == record.dept_id)
            .map(|child| dept_node(child, records))
            .collect(),
    }
}

fn is_root(record: &DeptOptionRecord, records: &[DeptOptionRecord]) -> bool {
    record.parent_id == "0" || !records.iter().any(|item| item.dept_id == record.parent_id)
}
