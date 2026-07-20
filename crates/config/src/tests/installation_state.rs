use std::fs;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use crate::{
    ConfigEncryptionKey, INSTALLATION_STATE_ENVELOPE_ALGORITHM, INSTALLATION_STATE_ENVELOPE_VERSION, InstallationStateError, InstallationStateRead,
    InstallationStateStore, decrypt_installation_state, encrypt_installation_state,
};

const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
struct TestState {
    database: String,
    retry_count: u8,
}

fn key() -> ConfigEncryptionKey {
    ConfigEncryptionKey::parse(KEY).unwrap()
}

fn state() -> TestState {
    TestState {
        database: "postgresql://installer:password@db/taco".into(),
        retry_count: 2,
    }
}

#[test]
fn encrypted_envelope_round_trips_generic_json_state() {
    let original = state();
    let envelope = encrypt_installation_state(&original, &key()).unwrap();
    let decoded: TestState = decrypt_installation_state(&envelope, &key()).unwrap();

    assert_eq!(envelope.version, INSTALLATION_STATE_ENVELOPE_VERSION);
    assert_eq!(envelope.algorithm, INSTALLATION_STATE_ENVELOPE_ALGORITHM);
    assert_eq!(URL_SAFE_NO_PAD.decode(&envelope.nonce).unwrap().len(), 24);
    assert_eq!(decoded, original);
}

#[test]
fn state_store_reports_absence_and_removes_state_idempotently() {
    let directory = tempdir().unwrap();
    let store = InstallationStateStore::new(directory.path());

    assert!(matches!(store.read::<TestState>(&key()).unwrap(), InstallationStateRead::Absent));
    store.write(&key(), &state()).unwrap();
    store.remove().unwrap();
    store.remove().unwrap();
    assert!(matches!(store.read::<TestState>(&key()).unwrap(), InstallationStateRead::Absent));
}

#[test]
fn state_store_writes_only_an_envelope_and_reads_the_latest_atomic_value() {
    let directory = tempdir().unwrap();
    let state_dir = directory.path().join("data");
    let store = InstallationStateStore::new(&state_dir);
    let mut replacement = state();
    replacement.retry_count = 7;

    store.write(&key(), &state()).unwrap();
    store.write(&key(), &replacement).unwrap();
    let persisted = fs::read(store.state_path()).unwrap();
    let loaded = match store.read::<TestState>(&key()).unwrap() {
        InstallationStateRead::Present(value) => value,
        InstallationStateRead::Absent => panic!("state file was written"),
    };

    assert!(!String::from_utf8_lossy(&persisted).contains("postgresql://installer"));
    assert_eq!(loaded, replacement);
    assert_eq!(fs::read_dir(state_dir).unwrap().count(), 1);
}

#[test]
fn present_malformed_or_tampered_state_is_an_explicit_error() {
    let directory = tempdir().unwrap();
    let store = InstallationStateStore::new(directory.path());
    fs::write(store.state_path(), b"not json").unwrap();
    assert!(matches!(store.read::<TestState>(&key()), Err(InstallationStateError::MalformedEnvelope)));

    let mut envelope = encrypt_installation_state(&state(), &key()).unwrap();
    envelope.ciphertext = URL_SAFE_NO_PAD.encode([0_u8; 16]);
    fs::write(store.state_path(), serde_json::to_vec(&envelope).unwrap()).unwrap();
    assert!(matches!(store.read::<TestState>(&key()), Err(InstallationStateError::AuthenticationFailed)));
}

#[test]
fn envelope_rejects_unsupported_metadata_before_decryption() {
    let mut envelope = encrypt_installation_state(&state(), &key()).unwrap();
    envelope.version = INSTALLATION_STATE_ENVELOPE_VERSION + 1;
    assert!(matches!(
        decrypt_installation_state::<TestState>(&envelope, &key()),
        Err(InstallationStateError::UnsupportedVersion(version)) if version == INSTALLATION_STATE_ENVELOPE_VERSION + 1
    ));

    envelope.version = INSTALLATION_STATE_ENVELOPE_VERSION;
    envelope.algorithm = "unknown".into();
    assert!(matches!(
        decrypt_installation_state::<TestState>(&envelope, &key()),
        Err(InstallationStateError::UnsupportedAlgorithm)
    ));
}
