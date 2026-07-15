use sqlx::{PgPool, query, query_scalar};

#[derive(Clone, Copy)]
pub(super) struct UserFixture<'a> {
    id: &'a str,
    username: &'a str,
    email: &'a str,
    phone: Option<&'a str>,
    del_flag: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct RoleFixture<'a> {
    id: &'a str,
    name: &'a str,
    key: &'a str,
    del_flag: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct MenuFixture<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub path: &'a str,
    pub route_name: &'a str,
}

#[derive(Clone, Copy)]
pub(super) struct RelationFixture<'a> {
    table: &'a str,
    column: &'a str,
    value: &'a str,
}

impl<'a> UserFixture<'a> {
    pub(super) const fn active(id: &'a str, username: &'a str, email: &'a str) -> Self {
        Self {
            id,
            username,
            email,
            phone: None,
            del_flag: "0",
        }
    }

    pub(super) const fn with_phone(self, phone: &'a str) -> Self {
        Self { phone: Some(phone), ..self }
    }

    pub(super) async fn insert(&self, pool: &PgPool) {
        self.insert_result(pool).await.unwrap();
    }

    pub(super) async fn insert_result(&self, pool: &PgPool) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        query("INSERT INTO sys_user (user_id,user_name,nick_name,email,phonenumber,password,del_flag,create_time) VALUES ($1,$2,$2,LOWER($3),$4,'hash',$5,CURRENT_TIMESTAMP)")
            .bind(self.id)
            .bind(self.username)
            .bind(self.email)
            .bind(self.phone)
            .bind(self.del_flag)
            .execute(pool)
            .await
    }
}

impl<'a> RoleFixture<'a> {
    pub(super) const fn active(id: &'a str, name: &'a str, key: &'a str) -> Self {
        Self { id, name, key, del_flag: "0" }
    }

    pub(super) const fn with_del_flag(self, del_flag: &'a str) -> Self {
        Self { del_flag, ..self }
    }

    pub(super) async fn insert(&self, pool: &PgPool) {
        self.insert_result(pool).await.unwrap();
    }

    pub(super) async fn insert_result(&self, pool: &PgPool) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,del_flag,create_time) VALUES ($1,$2,$3,1,'0',$4,CURRENT_TIMESTAMP)")
            .bind(self.id)
            .bind(self.name)
            .bind(self.key)
            .bind(self.del_flag)
            .execute(pool)
            .await
    }
}

impl MenuFixture<'_> {
    pub(super) async fn insert(&self, pool: &PgPool) {
        self.insert_result(pool).await.unwrap();
    }

    pub(super) async fn insert_result(&self, pool: &PgPool) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
        query("INSERT INTO sys_menu (menu_id,menu_name,parent_id,path,route_name,create_time) VALUES ($1,$2,'0',$3,$4,CURRENT_TIMESTAMP)")
            .bind(self.id)
            .bind(self.name)
            .bind(self.path)
            .bind(self.route_name)
            .execute(pool)
            .await
    }
}

impl<'a> RelationFixture<'a> {
    pub(super) const fn new(table: &'a str, column: &'a str, value: &'a str) -> Self {
        Self { table, column, value }
    }

    pub(super) async fn count(&self, pool: &PgPool) -> i64 {
        query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {} WHERE {}=$1", self.table, self.column)))
            .bind(self.value)
            .fetch_one(pool)
            .await
            .unwrap()
    }
}
