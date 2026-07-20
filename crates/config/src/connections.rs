use url::Url;

use crate::{DatabaseSettings, RedisSettings, Settings, SettingsError};

const STRUCTURED_URL_BASE: &str = "placeholder://localhost";

impl Settings {
    pub fn database_url(&self) -> Result<String, SettingsError> {
        self.database.url()
    }

    pub fn redis_url(&self) -> Result<String, SettingsError> {
        self.redis.url()
    }
}

impl DatabaseSettings {
    pub fn url(&self) -> Result<String, SettingsError> {
        validate_port("database.port", self.port)?;
        let host = required("database.host", &self.host)?;
        let username = required("database.username", &self.username)?;
        let name = required("database.name", &self.name)?;
        require_secret("database.password", &self.password)?;

        let mut url = structured_url(self.scheme.as_str(), "database.scheme")?;
        set_host(&mut url, host, "database.host")?;
        set_port(&mut url, self.port, "database.port")?;
        let username = escape_userinfo_percent(username);
        url.set_username(&username)
            .map_err(|_| SettingsError::InvalidUrlComponent("database.username"))?;
        let password = escape_userinfo_percent(&self.password);
        url.set_password(Some(&password))
            .map_err(|_| SettingsError::InvalidUrlComponent("database.password"))?;
        url.path_segments_mut()
            .map_err(|_| SettingsError::InvalidUrlComponent("database.name"))?
            .clear()
            .push(name);
        url.query_pairs_mut().append_pair("sslmode", self.ssl_mode.as_str());
        Ok(url.into())
    }
}

impl RedisSettings {
    pub fn url(&self) -> Result<String, SettingsError> {
        validate_port("redis.port", self.port)?;
        let host = required("redis.host", &self.host)?;
        validate_optional("redis.username", self.username.as_deref())?;
        validate_optional("redis.password", self.password.as_deref())?;
        required("redis.key_prefix", &self.key_prefix)?;

        let mut url = structured_url(self.scheme.as_str(), "redis.scheme")?;
        set_host(&mut url, host, "redis.host")?;
        set_port(&mut url, self.port, "redis.port")?;
        set_redis_auth(&mut url, self)?;
        set_redis_options(&mut url, self)?;
        Ok(url.into())
    }
}

fn structured_url(scheme: &str, key: &'static str) -> Result<Url, SettingsError> {
    let mut url = Url::parse(STRUCTURED_URL_BASE).map_err(|_| SettingsError::InvalidUrlComponent(key))?;
    url.set_scheme(scheme).map_err(|_| SettingsError::InvalidUrlComponent(key))?;
    Ok(url)
}

fn set_host(url: &mut Url, host: &str, key: &'static str) -> Result<(), SettingsError> {
    url.set_host(Some(host)).map_err(|_| SettingsError::InvalidUrlComponent(key))
}

fn set_port(url: &mut Url, port: u16, key: &'static str) -> Result<(), SettingsError> {
    url.set_port(Some(port)).map_err(|_| SettingsError::InvalidUrlComponent(key))
}

fn set_redis_auth(url: &mut Url, settings: &RedisSettings) -> Result<(), SettingsError> {
    if let Some(username) = settings.username.as_deref() {
        let username = escape_userinfo_percent(username.trim());
        url.set_username(&username).map_err(|_| SettingsError::InvalidUrlComponent("redis.username"))?;
    }
    if let Some(password) = settings.password.as_deref() {
        let password = escape_userinfo_percent(password);
        url.set_password(Some(&password))
            .map_err(|_| SettingsError::InvalidUrlComponent("redis.password"))?;
    }
    Ok(())
}

// Url userinfo setters preserve existing `%xx`, so escape raw percent signs first.
fn escape_userinfo_percent(value: &str) -> String {
    value.replace('%', "%25")
}

fn set_redis_options(url: &mut Url, settings: &RedisSettings) -> Result<(), SettingsError> {
    if let Some(database) = settings.database {
        url.path_segments_mut()
            .map_err(|_| SettingsError::InvalidUrlComponent("redis.database"))?
            .clear()
            .push(&database.to_string());
    }
    if let Some(protocol) = settings.protocol {
        url.query_pairs_mut().append_pair("protocol", protocol.as_str());
    }
    Ok(())
}

fn required<'a>(key: &'static str, value: &'a str) -> Result<&'a str, SettingsError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }
    Ok(value)
}

fn require_secret(key: &'static str, value: &str) -> Result<(), SettingsError> {
    required(key, value).map(|_| ())
}

fn validate_optional(key: &'static str, value: Option<&str>) -> Result<(), SettingsError> {
    match value {
        Some(value) => required(key, value).map(|_| ()),
        None => Ok(()),
    }
}

fn validate_port(key: &'static str, port: u16) -> Result<(), SettingsError> {
    if port == 0 {
        return Err(SettingsError::NonPositiveNumber(key));
    }
    Ok(())
}
