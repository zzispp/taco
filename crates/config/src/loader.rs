use crate::{
    EnvironmentReadError, EnvironmentReader, Settings, SettingsError,
    environment::ProcessEnvironment,
    interpolation::{InterpolatingDeserializer, InterpolationError, InterpolationErrorKind},
};
use config_rs::{Config, File, Value};
use sha2::{Digest, Sha256};
use std::{env, path::PathBuf};

const CONFIG_ARG: &str = "--config";
const KNOWN_INSECURE_JWT_SECRET_SHA256: [[u8; 32]; 2] = [
    [
        0xb8, 0x9f, 0x85, 0xb2, 0x25, 0x06, 0xeb, 0x72, 0xf6, 0x0b, 0x3a, 0x7b, 0x3c, 0xa1, 0xd0, 0x9b, 0x74, 0x90, 0xd0, 0xe0, 0x52, 0xe2, 0x08, 0xec, 0xfd,
        0xa4, 0x88, 0xe0, 0x7a, 0x09, 0x66, 0x8f,
    ],
    [
        0x33, 0xba, 0xdc, 0x0f, 0x05, 0x15, 0xad, 0xd7, 0x09, 0xe4, 0x25, 0xf5, 0x94, 0x57, 0xec, 0x0f, 0x96, 0x61, 0xf5, 0x65, 0x7b, 0xca, 0x54, 0xc7, 0x58,
        0x48, 0x78, 0x1c, 0x1d, 0x48, 0x77, 0x8f,
    ],
];
const MIN_JWT_SECRET_BYTES: usize = 32;
pub(crate) const FORBIDDEN_POSTGRES_ENVIRONMENT_VARIABLES: [&str; 13] = [
    "PGHOSTADDR",
    "PGHOST",
    "PGPORT",
    "PGUSER",
    "PGPASSWORD",
    "PGDATABASE",
    "PGSSLMODE",
    "PGSSLROOTCERT",
    "PGSSLCERT",
    "PGSSLKEY",
    "PGAPPNAME",
    "PGOPTIONS",
    "PGPASSFILE",
];
const DATABASE_CONFIG_PATH: &str = "database";

impl Settings {
    pub fn load() -> Result<Self, SettingsError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString>,
    {
        Self::load_from_args_with_environment(args, &ProcessEnvironment)
    }

    pub fn load_from_args_with_environment<I, S>(args: I, environment: &dyn EnvironmentReader) -> Result<Self, SettingsError>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<std::ffi::OsString>>();
        let path = explicit_config_path(&args)?;
        reject_postgres_environment(environment)?;
        let config = Config::builder().add_source(File::from(path)).build()?;
        let settings = deserialize_config_with_environment(config.cache, environment)?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn jwt_secret(&self) -> Result<String, SettingsError> {
        let secret = required_config_value("jwt.secret", &self.jwt.secret)?;
        if is_known_insecure_jwt_secret(&secret) {
            return Err(SettingsError::InsecureJwtSecret);
        }
        let actual_bytes = secret.len();
        if actual_bytes < MIN_JWT_SECRET_BYTES {
            return Err(SettingsError::JwtSecretTooShort {
                minimum_bytes: MIN_JWT_SECRET_BYTES,
                actual_bytes,
            });
        }
        Ok(secret)
    }

    pub fn cloudflare_turnstile_secret_key(&self) -> String {
        self.captcha.cloudflare_turnstile.secret_key.trim().to_owned()
    }
}

#[cfg(test)]
pub(crate) fn deserialize_settings_with_environment(source: &str, environment: &dyn EnvironmentReader) -> Result<Settings, SettingsError> {
    let config = Config::builder().add_source(File::from_str(source, config_rs::FileFormat::Yaml)).build()?;
    deserialize_config_with_environment(config.cache, environment)
}

fn deserialize_config_with_environment(value: Value, environment: &dyn EnvironmentReader) -> Result<Settings, SettingsError> {
    serde_path_to_error::deserialize(InterpolatingDeserializer::new(value, environment)).map_err(path_interpolation_error)
}

fn path_interpolation_error(error: serde_path_to_error::Error<InterpolationError>) -> SettingsError {
    let path = error.path().to_string();
    match error.into_inner().into_kind() {
        InterpolationErrorKind::Message(reason) => SettingsError::InvalidConfigValue { path, reason },
        InterpolationErrorKind::MissingEnvironmentVariable(variable) => SettingsError::MissingEnvironmentVariable { variable, path },
        InterpolationErrorKind::InvalidEnvironmentEncoding(variable) => SettingsError::InvalidEnvironmentEncoding { variable, path },
        InterpolationErrorKind::InvalidEnvironmentPlaceholder => SettingsError::InvalidEnvironmentPlaceholder { path },
        InterpolationErrorKind::InvalidEnvironmentValue { variable, expected } => SettingsError::InvalidEnvironmentValue { variable, path, expected },
    }
}

fn is_known_insecure_jwt_secret(secret: &str) -> bool {
    let digest: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
    KNOWN_INSECURE_JWT_SECRET_SHA256.contains(&digest)
}

pub(crate) fn reject_postgres_environment(environment: &dyn EnvironmentReader) -> Result<(), SettingsError> {
    for variable in FORBIDDEN_POSTGRES_ENVIRONMENT_VARIABLES {
        match environment.read(variable) {
            Ok(Some(_)) => return Err(SettingsError::ConflictingPostgresEnvironmentVariable(variable)),
            Ok(None) => {}
            Err(EnvironmentReadError::NotUnicode) => {
                return Err(SettingsError::InvalidEnvironmentEncoding {
                    variable: variable.into(),
                    path: DATABASE_CONFIG_PATH.into(),
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn explicit_config_path(args: &[std::ffi::OsString]) -> Result<PathBuf, SettingsError> {
    let index = args.iter().position(|arg| arg == CONFIG_ARG).ok_or(SettingsError::MissingConfigArgument)?;
    args.get(index + 1).map(PathBuf::from).ok_or(SettingsError::MissingConfigArgument)
}

pub(crate) fn required_config_value(key: &'static str, value: &str) -> Result<String, SettingsError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(SettingsError::BlankConfigValue(key));
    }

    Ok(trimmed.to_owned())
}
