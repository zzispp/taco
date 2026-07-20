use std::sync::Arc;

use configuration::{DatabaseSettings, InstallationProfile, PersistedInstallation, RedisSettings};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{ExistingInstallationVerifier, JwtSecretGenerator, SetupPortFailure, valid_jwt_secret};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InstallationConnections {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
}

#[derive(Clone)]
pub struct InstallationRecoveryService {
    verifier: Arc<dyn ExistingInstallationVerifier>,
    jwt_secret_generator: Arc<dyn JwtSecretGenerator>,
}

#[derive(Debug, Error)]
pub enum InstallationRecoveryError {
    #[error("the existing installation state is incomplete")]
    IncompleteState,
    #[error("existing installation verification failed: {0}")]
    Verification(#[from] super::ExistingInstallationVerificationFailure),
    #[error("JWT signing key generation failed")]
    JwtGenerationFailed,
    #[error("generated JWT signing key is invalid")]
    InvalidGeneratedJwt,
}

impl InstallationRecoveryService {
    pub fn new(verifier: Arc<dyn ExistingInstallationVerifier>, jwt_secret_generator: Arc<dyn JwtSecretGenerator>) -> Self {
        Self {
            verifier,
            jwt_secret_generator,
        }
    }

    pub async fn reconfigure(
        &self,
        installation: PersistedInstallation,
        connections: InstallationConnections,
    ) -> Result<PersistedInstallation, InstallationRecoveryError> {
        let mut profile = complete_profile(installation)?;
        profile.database = connections.database;
        profile.redis = connections.redis;
        self.verifier.verify_existing_installation(&profile).await?;
        Ok(PersistedInstallation::completed(profile))
    }

    pub async fn recover(&self, mut profile: InstallationProfile) -> Result<PersistedInstallation, InstallationRecoveryError> {
        self.verifier.verify_existing_installation(&profile).await?;
        profile.jwt.secret = self.generate_jwt_secret()?;
        Ok(PersistedInstallation::completed(profile))
    }

    fn generate_jwt_secret(&self) -> Result<String, InstallationRecoveryError> {
        let secret = self.jwt_secret_generator.generate_jwt_secret().map_err(map_jwt_failure)?;
        if !valid_jwt_secret(&secret) {
            return Err(InstallationRecoveryError::InvalidGeneratedJwt);
        }
        Ok(secret)
    }
}

fn complete_profile(installation: PersistedInstallation) -> Result<InstallationProfile, InstallationRecoveryError> {
    installation
        .complete
        .then_some(installation.profile)
        .ok_or(InstallationRecoveryError::IncompleteState)
}

fn map_jwt_failure(_: SetupPortFailure) -> InstallationRecoveryError {
    InstallationRecoveryError::JwtGenerationFailed
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use configuration::{InstallationProfile, PersistedInstallation};

    use super::{InstallationConnections, InstallationRecoveryService};
    use crate::application::{ExistingInstallationVerificationFailure, ExistingInstallationVerifier, JwtSecretGenerator, SetupPortFailure};

    const TEST_JWT_SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[tokio::test]
    async fn reconfigure_preserves_existing_non_connection_state_after_verification() {
        let verifier = Arc::new(Verifier::default());
        let service = service(verifier.clone());
        let mut current = InstallationProfile::default();
        current.jwt.secret = "existing-jwt-secret-with-32-characters".into();
        let mut updated = InstallationProfile::default();
        updated.database.host = "postgres.new.test".into();
        updated.redis.host = "redis.new.test".into();

        let result = service
            .reconfigure(
                PersistedInstallation::completed(current.clone()),
                InstallationConnections {
                    database: updated.database.clone(),
                    redis: updated.redis.clone(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.profile.database, updated.database);
        assert_eq!(result.profile.redis, updated.redis);
        assert_eq!(result.profile.jwt, current.jwt);
        assert_eq!(verifier.verified_hosts(), vec!["postgres.new.test"]);
    }

    #[tokio::test]
    async fn recover_verifies_before_rotating_the_jwt_secret() {
        let verifier = Arc::new(Verifier::default());
        let service = service(verifier.clone());
        let mut profile = InstallationProfile::default();
        profile.database.host = "postgres.recovery.test".into();
        profile.jwt.secret = "operator-provided-secret-is-replaced".into();

        let result = service.recover(profile).await.unwrap();

        assert_eq!(result.profile.jwt.secret, TEST_JWT_SECRET);
        assert_eq!(verifier.verified_hosts(), vec!["postgres.recovery.test"]);
    }

    fn service(verifier: Arc<Verifier>) -> InstallationRecoveryService {
        InstallationRecoveryService::new(verifier, Arc::new(SecretGenerator))
    }

    #[derive(Default)]
    struct Verifier {
        hosts: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl ExistingInstallationVerifier for Verifier {
        async fn verify_existing_installation(&self, profile: &InstallationProfile) -> Result<(), ExistingInstallationVerificationFailure> {
            self.hosts.lock().unwrap().push(profile.database.host.clone());
            Ok(())
        }
    }

    impl Verifier {
        fn verified_hosts(&self) -> Vec<String> {
            self.hosts.lock().unwrap().clone()
        }
    }

    struct SecretGenerator;

    impl JwtSecretGenerator for SecretGenerator {
        fn generate_jwt_secret(&self) -> Result<String, SetupPortFailure> {
            Ok(TEST_JWT_SECRET.into())
        }
    }
}
