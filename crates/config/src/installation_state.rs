use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit},
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::ConfigEncryptionKey;

pub const INSTALLATION_STATE_FILE_NAME: &str = "installation-state.enc";
pub const INSTALLATION_STATE_ENVELOPE_VERSION: u8 = 1;
pub const INSTALLATION_STATE_ENVELOPE_ALGORITHM: &str = "xchacha20poly1305";
const XCHACHA20_NONCE_BYTES: usize = 24;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationStateEnvelope {
    pub version: u8,
    pub algorithm: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub enum InstallationStateRead<T> {
    Absent,
    Present(T),
}

#[derive(Clone, Debug)]
pub struct InstallationStateStore {
    data_dir: PathBuf,
}

impl InstallationStateStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_owned(),
        }
    }

    pub fn state_path(&self) -> PathBuf {
        self.data_dir.join(INSTALLATION_STATE_FILE_NAME)
    }

    pub fn read<T>(&self, key: &ConfigEncryptionKey) -> Result<InstallationStateRead<T>, InstallationStateError>
    where
        T: DeserializeOwned,
    {
        let bytes = match fs::read(self.state_path()) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(InstallationStateRead::Absent),
            Err(error) => return Err(InstallationStateError::Read(error)),
        };
        let envelope = serde_json::from_slice(&bytes).map_err(|_| InstallationStateError::MalformedEnvelope)?;
        Ok(InstallationStateRead::Present(decrypt_installation_state(&envelope, key)?))
    }

    pub fn write<T>(&self, key: &ConfigEncryptionKey, state: &T) -> Result<(), InstallationStateError>
    where
        T: Serialize,
    {
        let envelope = encrypt_installation_state(state, key)?;
        let bytes = serde_json::to_vec(&envelope).map_err(InstallationStateError::Serialize)?;
        write_atomically(&self.data_dir, &self.state_path(), &bytes)
    }

    pub fn remove(&self) -> Result<(), InstallationStateError> {
        match fs::remove_file(self.state_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(InstallationStateError::Remove(error)),
        }
    }
}

pub fn encrypt_installation_state<T>(state: &T, key: &ConfigEncryptionKey) -> Result<InstallationStateEnvelope, InstallationStateError>
where
    T: Serialize,
{
    let plaintext = serde_json::to_vec(state).map_err(InstallationStateError::Serialize)?;
    let mut nonce = [0; XCHACHA20_NONCE_BYTES];
    OsRng.fill_bytes(&mut nonce);
    encrypt_with_nonce(&plaintext, key, nonce)
}

pub fn decrypt_installation_state<T>(envelope: &InstallationStateEnvelope, key: &ConfigEncryptionKey) -> Result<T, InstallationStateError>
where
    T: DeserializeOwned,
{
    let nonce = validate_envelope(envelope)?;
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| InstallationStateError::Encryption)?;
    let plaintext = cipher
        .decrypt(&XNonce::from(nonce), envelope_ciphertext(envelope)?.as_ref())
        .map_err(|_| InstallationStateError::AuthenticationFailed)?;
    serde_json::from_slice(&plaintext).map_err(InstallationStateError::Deserialize)
}

fn encrypt_with_nonce(
    plaintext: &[u8],
    key: &ConfigEncryptionKey,
    nonce: [u8; XCHACHA20_NONCE_BYTES],
) -> Result<InstallationStateEnvelope, InstallationStateError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key.as_bytes()).map_err(|_| InstallationStateError::Encryption)?;
    let ciphertext = cipher
        .encrypt(&XNonce::from(nonce), plaintext)
        .map_err(|_| InstallationStateError::Encryption)?;
    Ok(InstallationStateEnvelope {
        version: INSTALLATION_STATE_ENVELOPE_VERSION,
        algorithm: INSTALLATION_STATE_ENVELOPE_ALGORITHM.into(),
        nonce: URL_SAFE_NO_PAD.encode(nonce),
        ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
    })
}

fn validate_envelope(envelope: &InstallationStateEnvelope) -> Result<[u8; XCHACHA20_NONCE_BYTES], InstallationStateError> {
    if envelope.version != INSTALLATION_STATE_ENVELOPE_VERSION {
        return Err(InstallationStateError::UnsupportedVersion(envelope.version));
    }
    if envelope.algorithm != INSTALLATION_STATE_ENVELOPE_ALGORITHM {
        return Err(InstallationStateError::UnsupportedAlgorithm);
    }
    decode_fixed(&envelope.nonce)
}

fn envelope_ciphertext(envelope: &InstallationStateEnvelope) -> Result<Vec<u8>, InstallationStateError> {
    URL_SAFE_NO_PAD
        .decode(&envelope.ciphertext)
        .map_err(|_| InstallationStateError::MalformedEnvelope)
}

fn decode_fixed(value: &str) -> Result<[u8; XCHACHA20_NONCE_BYTES], InstallationStateError> {
    let bytes = URL_SAFE_NO_PAD.decode(value).map_err(|_| InstallationStateError::MalformedEnvelope)?;
    bytes.try_into().map_err(|_| InstallationStateError::MalformedEnvelope)
}

fn write_atomically(data_dir: &Path, state_path: &Path, bytes: &[u8]) -> Result<(), InstallationStateError> {
    fs::create_dir_all(data_dir).map_err(InstallationStateError::CreateDataDirectory)?;
    let mut temporary = NamedTempFile::new_in(data_dir).map_err(InstallationStateError::CreateTemporaryFile)?;
    temporary.write_all(bytes).map_err(InstallationStateError::WriteTemporaryFile)?;
    temporary.as_file().sync_all().map_err(InstallationStateError::SyncTemporaryFile)?;
    temporary.persist(state_path).map_err(|error| InstallationStateError::Replace(error.error))?;
    sync_data_directory(data_dir)
}

fn sync_data_directory(data_dir: &Path) -> Result<(), InstallationStateError> {
    let directory = File::open(data_dir).map_err(InstallationStateError::OpenDataDirectory)?;
    directory.sync_all().map_err(InstallationStateError::SyncDataDirectory)
}

#[derive(Debug, Error)]
pub enum InstallationStateError {
    #[error("failed to serialize installation state")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to encrypt installation state")]
    Encryption,
    #[error("installation state envelope is malformed")]
    MalformedEnvelope,
    #[error("installation state envelope version {0} is unsupported")]
    UnsupportedVersion(u8),
    #[error("installation state envelope algorithm is unsupported")]
    UnsupportedAlgorithm,
    #[error("failed to authenticate installation state")]
    AuthenticationFailed,
    #[error("failed to deserialize installation state")]
    Deserialize(#[source] serde_json::Error),
    #[error("failed to read installation state file")]
    Read(#[source] io::Error),
    #[error("failed to create installation data directory")]
    CreateDataDirectory(#[source] io::Error),
    #[error("failed to create temporary installation state file")]
    CreateTemporaryFile(#[source] io::Error),
    #[error("failed to write temporary installation state file")]
    WriteTemporaryFile(#[source] io::Error),
    #[error("failed to synchronize temporary installation state file")]
    SyncTemporaryFile(#[source] io::Error),
    #[error("failed to atomically replace installation state file")]
    Replace(#[source] io::Error),
    #[error("failed to open installation data directory for synchronization")]
    OpenDataDirectory(#[source] io::Error),
    #[error("failed to synchronize installation data directory")]
    SyncDataDirectory(#[source] io::Error),
    #[error("failed to remove installation state file")]
    Remove(#[source] io::Error),
}
