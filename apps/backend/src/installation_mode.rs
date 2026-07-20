use configuration::{BootstrapInputs, InstallationStateRead, InstallationStateStore, Settings};

use crate::BackendResult;

pub(crate) enum InstallationMode {
    Setup(BootstrapInputs),
    Normal(Box<Settings>),
}

pub(crate) fn classify(bootstrap: BootstrapInputs) -> BackendResult<InstallationMode> {
    let store = InstallationStateStore::new(&bootstrap.data_dir);
    let installation = store.read(&bootstrap.config_encryption_key)?;
    match installation {
        InstallationStateRead::Absent => Ok(InstallationMode::Setup(bootstrap)),
        InstallationStateRead::Present(installation) => {
            let settings = Settings::from_persisted_installation(installation, &bootstrap)?;
            Ok(InstallationMode::Normal(Box::new(settings)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use configuration::{BootstrapInputs, ConfigEncryptionKey, DataDirectory, InstallationProfile, InstallationStateStore, PersistedInstallation};

    use super::{InstallationMode, classify};

    const TEST_ROOT_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    #[test]
    fn missing_state_enters_setup_mode() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mode = classify(bootstrap(temp_dir.path().to_owned())).unwrap();

        assert!(matches!(mode, InstallationMode::Setup(_)));
    }

    #[test]
    fn complete_state_enters_normal_mode_with_derived_settings() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bootstrap = bootstrap(temp_dir.path().to_owned());
        let profile = valid_profile();
        InstallationStateStore::new(&bootstrap.data_dir)
            .write(&bootstrap.config_encryption_key, &PersistedInstallation::completed(profile))
            .unwrap();

        let mode = classify(bootstrap).unwrap();
        let InstallationMode::Normal(settings) = mode else {
            panic!("complete installation state must enter normal mode");
        };

        assert_eq!(settings.bind_addr(), "0.0.0.0:3000");
        assert!(settings.uploads.avatar_directory.ends_with("uploads/avatars"));
    }

    #[test]
    fn incomplete_state_fails_instead_of_reopening_setup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let bootstrap = bootstrap(temp_dir.path().to_owned());
        InstallationStateStore::new(&bootstrap.data_dir)
            .write(
                &bootstrap.config_encryption_key,
                &PersistedInstallation {
                    complete: false,
                    profile: valid_profile(),
                },
            )
            .unwrap();

        let error = match classify(bootstrap) {
            Ok(_) => panic!("incomplete installation state must fail startup"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("installation state is not complete"));
    }

    fn bootstrap(data_dir: PathBuf) -> BootstrapInputs {
        BootstrapInputs::new(
            DataDirectory::new(data_dir).unwrap(),
            ConfigEncryptionKey::parse(TEST_ROOT_KEY).unwrap(),
            "0.0.0.0:3000".parse().unwrap(),
        )
    }

    fn valid_profile() -> InstallationProfile {
        let mut profile = InstallationProfile::default();
        profile.database.host = "database.example.test".into();
        profile.database.username = "taco".into();
        profile.database.password = "database-password".into();
        profile.database.name = "taco".into();
        profile.jwt.secret = "x".repeat(32);
        profile.redis.host = "redis.example.test".into();
        profile
    }
}
