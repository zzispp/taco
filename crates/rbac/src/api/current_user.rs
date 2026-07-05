#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub role_keys: Vec<String>,
    pub permissions: Vec<String>,
    pub dept_id: Option<String>,
    pub admin: bool,
    pub system: bool,
}
