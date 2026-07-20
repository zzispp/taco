mod advanced_setup;
mod setup_input;
mod state;

pub use advanced_setup::AdvancedSetupOverrides;
pub use setup_input::{
    InitialAdministrator, InitialAdministratorInput, PostgresConnection, PostgresConnectionInput, RedisConnection, RedisConnectionInput, SetupInputError,
};
pub use state::InstallationState;

#[cfg(test)]
mod tests;
