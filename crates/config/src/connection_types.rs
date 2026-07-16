use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseScheme {
    Postgres,
    Postgresql,
}

impl DatabaseScheme {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Postgresql => "postgresql",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatabaseSslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl DatabaseSslMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Allow => "allow",
            Self::Prefer => "prefer",
            Self::Require => "require",
            Self::VerifyCa => "verify-ca",
            Self::VerifyFull => "verify-full",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedisScheme {
    Redis,
    Rediss,
}

impl RedisScheme {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Redis => "redis",
            Self::Rediss => "rediss",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedisProtocol {
    Resp2,
    Resp3,
}

impl RedisProtocol {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Resp2 => "resp2",
            Self::Resp3 => "resp3",
        }
    }
}
