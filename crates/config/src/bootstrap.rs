use std::{
    env,
    ffi::OsString,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use thiserror::Error;

use crate::{EnvironmentReadError, EnvironmentReader, environment::ProcessEnvironment};

pub const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:3000";
const CONFIG_ENCRYPTION_KEY_BYTES: usize = 32;
const DATA_DIR_ARGUMENT: &str = "--data-dir";
const DATA_DIR_ENVIRONMENT_VARIABLE: &str = "TACO_DATA_DIR";
const CONFIG_ENCRYPTION_KEY_ARGUMENT: &str = "--config-encryption-key";
const CONFIG_ENCRYPTION_KEY_ENVIRONMENT_VARIABLE: &str = "TACO_CONFIG_ENCRYPTION_KEY";
const LISTEN_ARGUMENT: &str = "--listen";
const LISTEN_ENVIRONMENT_VARIABLE: &str = "TACO_LISTEN_ADDR";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataDirectory(PathBuf);

impl DataDirectory {
    pub fn new(path: PathBuf) -> Result<Self, BootstrapInputError> {
        if path.as_os_str().is_empty() || path.to_string_lossy().trim().is_empty() {
            return Err(BootstrapInputError::BlankInput(DATA_DIR_ARGUMENT));
        }
        Ok(Self(path))
    }

    pub fn load() -> Result<Self, BootstrapInputError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self::load_from_args_with_environment(args, &ProcessEnvironment)
    }

    pub fn load_from_args_with_environment<I, S>(args: I, environment: &dyn EnvironmentReader) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args = collect_args(args);
        let source = resolve_source(&args, DATA_DIR_ARGUMENT, DATA_DIR_ENVIRONMENT_VARIABLE, environment)?;
        let source = source.ok_or(BootstrapInputError::MissingInput(DATA_DIR_ARGUMENT))?;
        Self::new(source.into_path())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn into_path(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for DataDirectory {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ConfigEncryptionKey([u8; CONFIG_ENCRYPTION_KEY_BYTES]);

impl ConfigEncryptionKey {
    pub fn generate() -> Self {
        let mut bytes = [0; CONFIG_ENCRYPTION_KEY_BYTES];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn parse(value: &str) -> Result<Self, BootstrapInputError> {
        let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| BootstrapInputError::InvalidConfigEncryptionKey)?;
        let bytes: [u8; CONFIG_ENCRYPTION_KEY_BYTES] = bytes.try_into().map_err(|_| BootstrapInputError::InvalidConfigEncryptionKey)?;
        Ok(Self(bytes))
    }

    pub fn encode(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn load() -> Result<Self, BootstrapInputError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self::load_from_args_with_environment(args, &ProcessEnvironment)
    }

    pub fn load_from_args_with_environment<I, S>(args: I, environment: &dyn EnvironmentReader) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args = collect_args(args);
        let source = resolve_source(&args, CONFIG_ENCRYPTION_KEY_ARGUMENT, CONFIG_ENCRYPTION_KEY_ENVIRONMENT_VARIABLE, environment)?;
        let source = source.ok_or(BootstrapInputError::MissingInput(CONFIG_ENCRYPTION_KEY_ARGUMENT))?;
        Self::parse(&source.into_string(CONFIG_ENCRYPTION_KEY_ARGUMENT)?)
    }

    pub(crate) fn as_bytes(&self) -> &[u8; CONFIG_ENCRYPTION_KEY_BYTES] {
        &self.0
    }
}

pub struct BootstrapInputs {
    pub data_dir: DataDirectory,
    pub config_encryption_key: ConfigEncryptionKey,
    pub listen_addr: SocketAddr,
}

impl BootstrapInputs {
    pub const fn new(data_dir: DataDirectory, config_encryption_key: ConfigEncryptionKey, listen_addr: SocketAddr) -> Self {
        Self {
            data_dir,
            config_encryption_key,
            listen_addr,
        }
    }

    pub fn load() -> Result<Self, BootstrapInputError> {
        Self::load_from_args(env::args_os())
    }

    pub fn load_from_args<I, S>(args: I) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self::load_from_args_with_environment(args, &ProcessEnvironment)
    }

    pub fn load_from_args_with_environment<I, S>(args: I, environment: &dyn EnvironmentReader) -> Result<Self, BootstrapInputError>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args = collect_args(args);
        let data_dir = DataDirectory::load_from_args_with_environment(args.clone(), environment)?;
        let config_encryption_key = ConfigEncryptionKey::load_from_args_with_environment(args.clone(), environment)?;
        let listen_addr = load_listen_addr(&args, environment)?;
        Ok(Self::new(data_dir, config_encryption_key, listen_addr))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BootstrapInputError {
    #[error("{0} is required")]
    MissingInput(&'static str),
    #[error("{0} requires a value")]
    MissingArgumentValue(&'static str),
    #[error("{0} can only be supplied once")]
    RepeatedArgument(&'static str),
    #[error("{argument} conflicts with {environment_variable}")]
    ConflictingSources {
        argument: &'static str,
        environment_variable: &'static str,
    },
    #[error("{0} cannot be blank")]
    BlankInput(&'static str),
    #[error("{0} is not valid UTF-8")]
    InvalidArgumentEncoding(&'static str),
    #[error("{0} is not valid UTF-8")]
    InvalidEnvironmentEncoding(&'static str),
    #[error("configuration encryption key must be a 32-byte Base64URL value")]
    InvalidConfigEncryptionKey,
    #[error("listen address must be a valid IP address and port")]
    InvalidListenAddress,
}

enum InputSource {
    Argument(OsString),
    Environment(String),
}

impl InputSource {
    fn into_path(self) -> PathBuf {
        match self {
            Self::Argument(value) => PathBuf::from(value),
            Self::Environment(value) => PathBuf::from(value),
        }
    }

    fn into_string(self, argument: &'static str) -> Result<String, BootstrapInputError> {
        match self {
            Self::Argument(value) => value.into_string().map_err(|_| BootstrapInputError::InvalidArgumentEncoding(argument)),
            Self::Environment(value) => Ok(value),
        }
    }
}

fn collect_args<I, S>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    args.into_iter().map(Into::into).collect()
}

fn resolve_source(
    args: &[OsString],
    argument: &'static str,
    environment_variable: &'static str,
    environment: &dyn EnvironmentReader,
) -> Result<Option<InputSource>, BootstrapInputError> {
    let argument_value = argument_value(args, argument)?;
    let environment_value = environment
        .read(environment_variable)
        .map_err(|error| environment_error(environment_variable, error))?;
    if argument_value.is_some() && environment_value.is_some() {
        return Err(BootstrapInputError::ConflictingSources {
            argument,
            environment_variable,
        });
    }
    Ok(argument_value
        .map(InputSource::Argument)
        .or_else(|| environment_value.map(InputSource::Environment)))
}

fn argument_value(args: &[OsString], argument: &'static str) -> Result<Option<OsString>, BootstrapInputError> {
    let mut indexes = args.iter().enumerate().filter_map(|(index, value)| (value == argument).then_some(index));
    let Some(index) = indexes.next() else {
        return Ok(None);
    };
    if indexes.next().is_some() {
        return Err(BootstrapInputError::RepeatedArgument(argument));
    }
    args.get(index + 1)
        .cloned()
        .ok_or(BootstrapInputError::MissingArgumentValue(argument))
        .map(Some)
}

fn environment_error(variable: &'static str, _error: EnvironmentReadError) -> BootstrapInputError {
    BootstrapInputError::InvalidEnvironmentEncoding(variable)
}

fn load_listen_addr(args: &[OsString], environment: &dyn EnvironmentReader) -> Result<SocketAddr, BootstrapInputError> {
    let source = resolve_source(args, LISTEN_ARGUMENT, LISTEN_ENVIRONMENT_VARIABLE, environment)?;
    let Some(source) = source else {
        return DEFAULT_LISTEN_ADDR.parse().map_err(|_| BootstrapInputError::InvalidListenAddress);
    };
    source
        .into_string(LISTEN_ARGUMENT)?
        .parse()
        .map_err(|_| BootstrapInputError::InvalidListenAddress)
}
