use std::fmt;

use thiserror::Error;

#[derive(Clone, PartialEq, Eq)]
pub struct PostgresConnection {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    use_tls: bool,
}

impl PostgresConnection {
    pub fn new(input: PostgresConnectionInput) -> Result<Self, SetupInputError> {
        Ok(Self {
            host: required_text("postgres.host", input.host)?,
            port: positive_port("postgres.port", input.port)?,
            username: required_text("postgres.username", input.username)?,
            password: required_secret("postgres.password", input.password)?,
            database: required_text("postgres.database", input.database)?,
            use_tls: input.use_tls,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub const fn use_tls(&self) -> bool {
        self.use_tls
    }
}

impl fmt::Debug for PostgresConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresConnection")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("database", &self.database)
            .field("use_tls", &self.use_tls)
            .finish()
    }
}

pub struct PostgresConnectionInput {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub use_tls: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RedisConnection {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    database: Option<u16>,
    use_tls: bool,
}

impl RedisConnection {
    pub fn new(input: RedisConnectionInput) -> Result<Self, SetupInputError> {
        Ok(Self {
            host: required_text("redis.host", input.host)?,
            port: positive_port("redis.port", input.port)?,
            username: optional_text(input.username),
            password: optional_secret(input.password),
            database: input.database,
            use_tls: input.use_tls,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    pub const fn database(&self) -> Option<u16> {
        self.database
    }

    pub const fn use_tls(&self) -> bool {
        self.use_tls
    }
}

impl fmt::Debug for RedisConnection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RedisConnection")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "[REDACTED]"))
            .field("database", &self.database)
            .field("use_tls", &self.use_tls)
            .finish()
    }
}

pub struct RedisConnectionInput {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub database: Option<u16>,
    pub use_tls: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub struct InitialAdministrator {
    username: String,
    email: String,
    password: String,
}

impl InitialAdministrator {
    pub fn new(input: InitialAdministratorInput) -> Result<Self, SetupInputError> {
        Ok(Self {
            username: required_text("administrator.username", input.username)?,
            email: required_text("administrator.email", input.email)?,
            password: required_secret("administrator.password", input.password)?,
        })
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl fmt::Debug for InitialAdministrator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InitialAdministrator")
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

pub struct InitialAdministratorInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SetupInputError {
    #[error("{0} cannot be blank")]
    BlankField(&'static str),
    #[error("{0} must be greater than zero")]
    NonPositiveNumber(&'static str),
}

fn required_text(field: &'static str, value: String) -> Result<String, SetupInputError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SetupInputError::BlankField(field));
    }
    Ok(trimmed.to_owned())
}

fn required_secret(field: &'static str, value: String) -> Result<String, SetupInputError> {
    if value.trim().is_empty() {
        return Err(SetupInputError::BlankField(field));
    }
    Ok(value)
}

fn optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn optional_secret(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn positive_port(field: &'static str, value: u16) -> Result<u16, SetupInputError> {
    (value != 0).then_some(value).ok_or(SetupInputError::NonPositiveNumber(field))
}
