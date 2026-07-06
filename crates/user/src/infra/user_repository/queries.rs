use constants::pagination::PAGE_INDEX_OFFSET;
use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{query, query_as, query_scalar};
use storage::{
    Database, StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;
use types::{
    rbac::{DataScopeFilter, RoleOption},
    system::{Post, TreeSelectNode},
    user::{ProfileUpdate, User, UserFormOptions, UserId, UserProfileGroups},
};

use crate::application::{ReplaceUserRecord, UserListFilter};

use super::{
    mapping::{UserRelations, role_summary, user},
    record::{RoleSummaryRecord, UserRecord},
    sql, write,
};

#[derive(Clone)]
pub struct UserQueries {
    database: Database,
}

impl UserQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, input: ReplaceUserRecord) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        let user_id = self.database.next_id();
        let password_hash = required_password(input.password_hash.clone())?;
        self.insert_user(&user_id, input, password_hash).await?;
        self.find_by_id(UserId(user_id)).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: UserId, input: ReplaceUserRecord) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        self.update_user(&id, input).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
    }

    pub async fn delete_many(&self, ids: Vec<UserId>) -> StorageResult<()> {
        let ids: Vec<String> = ids.into_iter().map(|id| id.0).collect();
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = ANY($1) AND del_flag = '0'")
            .bind(&ids)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *tx)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record("user_id = $1", id.0.as_str()).await
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record("email = $1", email).await
    }

    pub async fn find_by_phone(&self, phone: &str) -> StorageResult<Option<User>> {
        self.find_record("phonenumber = $1", phone).await
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_name = $1", username).await
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("email = $1", email).await
    }

    pub async fn find_auth_by_id(&self, id: UserId) -> StorageResult<Option<(User, String)>> {
        self.find_auth_record("user_id = $1", &id.0).await
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let now = OffsetDateTime::now_utc();
        let result = query("UPDATE sys_user SET login_date = $2, update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(now)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
    }

    pub async fn list(&self, filter: UserListFilter) -> StorageResult<Page<User>> {
        let page = filter.page;
        let request = PageSliceRequest {
            offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        };
        self.list_slice(filter, request).await
    }

    pub async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> StorageResult<Page<User>> {
        let page = filter.page;
        let offset = (page.page - PAGE_INDEX_OFFSET) * page.page_size;
        let ids = self.scoped_user_ids(&filter, &scope, page.page_size, offset).await?;
        let total = self.scoped_user_total(&filter, &scope).await?;
        Ok(Page {
            items: self.users_by_ids(ids).await?,
            total: to_u64(total)?,
            page: page.page,
            page_size: page.page_size,
        })
    }

    pub async fn update_password(&self, id: UserId, password_hash: String) -> StorageResult<()> {
        let result = query("UPDATE sys_user SET password=$2,pwd_update_date=CURRENT_TIMESTAMP,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(id.0)
            .bind(password_hash)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())
    }

    pub async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> StorageResult<User> {
        let result = query("UPDATE sys_user SET nick_name=$2,email=$3,phonenumber=$4,sex=$5,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(profile.nick_name)
            .bind(profile.email)
            .bind(profile.phonenumber)
            .bind(profile.sex)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_avatar(&self, id: UserId, avatar: String) -> StorageResult<User> {
        let result = query("UPDATE sys_user SET avatar=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(avatar)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_status(&self, id: UserId, status: String) -> StorageResult<User> {
        let result = query("UPDATE sys_user SET status=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(status)
            .execute(self.database.pool())
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> StorageResult<User> {
        write::ensure_ids_exist(self.database.pool(), "sys_role", "role_id", &role_ids).await?;
        let mut tx = self.database.pool().begin().await?;
        write::replace_roles(&mut tx, &id.0, role_ids).await?;
        tx.commit().await.map_err(StorageError::from)?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn profile_groups(&self, id: UserId) -> StorageResult<UserProfileGroups> {
        Ok(UserProfileGroups {
            role_group: self.role_group(&id.0).await?,
            post_group: self.post_group(&id.0).await?,
            dept_name: self.dept_name(&id.0).await?,
        })
    }

    pub async fn form_options(&self) -> StorageResult<UserFormOptions> {
        Ok(UserFormOptions {
            roles: self.role_options().await?,
            posts: self.post_options().await?,
            depts: dept_tree(self.dept_options().await?),
        })
    }

    pub async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> StorageResult<Page<User>> {
        let total = self.filtered_total(&filter).await?;
        let records = self.filtered_records_slice(&filter, request.limit, request.offset).await?;
        Ok(Page {
            items: self.users(records).await?,
            total: to_u64(total)?,
            page: request.page,
            page_size: request.page_size,
        })
    }

    async fn insert_user(&self, user_id: &str, input: ReplaceUserRecord, password_hash: String) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let now = OffsetDateTime::now_utc();
        query(sql::insert_user())
            .bind(user_id)
            .bind(input.dept_id)
            .bind(input.username)
            .bind(input.nick_name)
            .bind(input.email)
            .bind(input.phonenumber)
            .bind(input.sex)
            .bind(password_hash)
            .bind(input.status)
            .bind(input.remark)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        write::replace_relations(&mut tx, user_id, input.role_ids, input.post_ids).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    async fn update_user(&self, id: &UserId, input: ReplaceUserRecord) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        write::execute_user_update(&mut tx, id, &input).await?;
        write::replace_relations(&mut tx, &id.0, input.role_ids, input.post_ids).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    async fn users(&self, records: Vec<UserRecord>) -> StorageResult<Vec<User>> {
        let mut users = Vec::with_capacity(records.len());
        for record in records {
            let relations = self.relations(&record.user_id).await?;
            users.push(user(record, relations));
        }
        Ok(users)
    }

    async fn users_by_ids(&self, ids: Vec<String>) -> StorageResult<Vec<User>> {
        let mut users = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(user) = self.find_by_id(UserId(id)).await? {
                users.push(user);
            }
        }
        Ok(users)
    }

    async fn filtered_total(&self, filter: &UserListFilter) -> StorageResult<i64> {
        query_scalar(sql::filtered_users_total())
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn filtered_records_slice(&self, filter: &UserListFilter, limit: u64, offset: u64) -> StorageResult<Vec<UserRecord>> {
        let sql = sql::filtered_users("ORDER BY u.create_time ASC LIMIT $7 OFFSET $8");
        query_as::<_, UserRecord>(&sql)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(to_i64(limit)?)
            .bind(to_i64(offset)?)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn scoped_user_ids(&self, filter: &UserListFilter, scope: &DataScopeFilter, limit: u64, offset: u64) -> StorageResult<Vec<String>> {
        query_scalar(sql::scoped_user_ids())
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(to_i64(limit)?)
            .bind(to_i64(offset)?)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn scoped_user_total(&self, filter: &UserListFilter, scope: &DataScopeFilter) -> StorageResult<i64> {
        query_scalar(sql::scoped_user_total())
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(&filter.status)
            .bind(&filter.dept_id)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn find_record(&self, predicate: &str, value: &str) -> StorageResult<Option<User>> {
        let record = self.raw_record(predicate, value).await?;
        match record {
            Some(record) => self.user(record).await.map(Some),
            None => Ok(None),
        }
    }

    async fn find_auth_record(&self, predicate: &str, value: &str) -> StorageResult<Option<(User, String)>> {
        let record = self.raw_record(predicate, value).await?;
        match record {
            Some(record) => self.auth_user(record).await.map(Some),
            None => Ok(None),
        }
    }

    async fn raw_record(&self, predicate: &str, value: &str) -> StorageResult<Option<UserRecord>> {
        query_as::<_, UserRecord>(&format!("SELECT {} FROM sys_user WHERE del_flag = '0' AND {predicate}", sql::USER_COLUMNS))
            .bind(value)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn auth_user(&self, record: UserRecord) -> StorageResult<(User, String)> {
        let password = record.password.clone();
        Ok((self.user(record).await?, password))
    }

    async fn user(&self, record: UserRecord) -> StorageResult<User> {
        let relations = self.relations(&record.user_id).await?;
        Ok(user(record, relations))
    }

    async fn relations(&self, user_id: &str) -> StorageResult<UserRelations> {
        let roles = self.roles(user_id).await?;
        Ok(UserRelations {
            role_ids: roles.iter().map(|role| role.role_id.clone()).collect(),
            roles,
            post_ids: self.post_ids(user_id).await?,
            permissions: self.permissions(user_id).await?,
        })
    }

    async fn roles(&self, user_id: &str) -> StorageResult<Vec<types::rbac::RoleSummary>> {
        query_as::<_, RoleSummaryRecord>(sql::role_query())
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map(|records| records.into_iter().map(role_summary).collect())
            .map_err(StorageError::from)
    }

    async fn role_group(&self, user_id: &str) -> StorageResult<String> {
        query_scalar(sql::role_group_query())
            .bind(user_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn post_group(&self, user_id: &str) -> StorageResult<String> {
        query_scalar(sql::post_group_query())
            .bind(user_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn dept_name(&self, user_id: &str) -> StorageResult<Option<String>> {
        query_scalar(sql::dept_name_query())
            .bind(user_id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn post_ids(&self, user_id: &str) -> StorageResult<Vec<String>> {
        query_scalar("SELECT post_id FROM sys_user_post WHERE user_id = $1 ORDER BY post_id ASC")
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    async fn permissions(&self, user_id: &str) -> StorageResult<Vec<String>> {
        query_scalar(sql::permission_query())
            .bind(user_id)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
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

    async fn ensure_references(&self, input: &ReplaceUserRecord) -> StorageResult<()> {
        write::ensure_dept_exists(self.database.pool(), input.dept_id.as_deref()).await?;
        write::ensure_ids_exist(self.database.pool(), "sys_role", "role_id", &input.role_ids).await?;
        write::ensure_ids_exist(self.database.pool(), "sys_post", "post_id", &input.post_ids).await
    }
}

fn required_password(password_hash: Option<String>) -> StorageResult<String> {
    password_hash.ok_or_else(|| StorageError::Database("password_hash is required".into()))
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

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
