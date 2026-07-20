use std::fmt;

use crate::domain::{AdvancedSetupOverrides, InitialAdministrator, PostgresConnection, RedisConnection, SetupInputError};

pub struct SetupInstallationInput {
    postgres: PostgresConnection,
    redis: RedisConnection,
    administrator: InitialAdministrator,
    advanced: AdvancedSetupOverrides,
}

impl SetupInstallationInput {
    pub fn new(parts: SetupInstallationInputParts) -> Result<Self, SetupInputError> {
        Ok(Self {
            postgres: parts.postgres,
            redis: parts.redis,
            administrator: parts.administrator,
            advanced: parts.advanced.validate()?,
        })
    }

    pub(crate) fn into_parts(self) -> SetupInstallationInputParts {
        SetupInstallationInputParts {
            postgres: self.postgres,
            redis: self.redis,
            administrator: self.administrator,
            advanced: self.advanced,
        }
    }
}

impl fmt::Debug for SetupInstallationInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SetupInstallationInput")
            .field("postgres", &self.postgres)
            .field("redis", &self.redis)
            .field("administrator", &self.administrator)
            .field("advanced", &self.advanced)
            .finish()
    }
}

pub struct SetupInstallationInputParts {
    pub postgres: PostgresConnection,
    pub redis: RedisConnection,
    pub administrator: InitialAdministrator,
    pub advanced: AdvancedSetupOverrides,
}
