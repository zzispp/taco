use std::io::Write;

use configuration::{BootstrapInputs, ConfigEncryptionKey, DataDirectory, InstallationStateStore, Settings};
use storage::connect_database;

use crate::{
    BackendResult,
    installation_mode::{InstallationMode, classify},
    migration, startup,
};

mod migration_command;
mod parser;

use parser::{BackendCommand, InstallationCommand, MigrationCommand, SecretCommand, command_from_args};

pub async fn run() -> BackendResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match command_from_args(args.clone())? {
        BackendCommand::Serve => run_server(args).await,
        BackendCommand::Migration(command) => run_migration(installed_settings(args)?, command).await,
        BackendCommand::Secrets(SecretCommand::Generate) => write_generated_secret(),
        BackendCommand::Installation(InstallationCommand::Reset) => reset_installation_state(args),
    }
}

async fn run_server(args: Vec<String>) -> BackendResult<()> {
    match classify(BootstrapInputs::load_from_args(args)?)? {
        InstallationMode::Setup(bootstrap) => startup::serve_setup(bootstrap).await,
        InstallationMode::Normal(settings) => startup::serve(*settings).await,
    }
}

fn installed_settings(args: Vec<String>) -> BackendResult<Settings> {
    match classify(BootstrapInputs::load_from_args(args)?)? {
        InstallationMode::Normal(settings) => Ok(*settings),
        InstallationMode::Setup(_) => Err("installation is not complete; complete web setup before running migrations".into()),
    }
}

fn write_generated_secret() -> BackendResult<()> {
    let key = ConfigEncryptionKey::generate();
    writeln!(std::io::stdout().lock(), "TACO_CONFIG_ENCRYPTION_KEY={}", key.encode())?;
    Ok(())
}

fn reset_installation_state(args: Vec<String>) -> BackendResult<()> {
    let data_dir = DataDirectory::load_from_args(args)?;
    InstallationStateStore::new(data_dir).remove()?;
    Ok(())
}

async fn run_migration(settings: Settings, command: MigrationCommand) -> BackendResult<()> {
    let database = connect_database(&settings.database_url()?).await?;
    match command {
        MigrationCommand::Up => migration::up(database.raw_pool(), None).await?,
        MigrationCommand::Status => print_status(migration::status(database.raw_pool()).await?)?,
    }
    Ok(())
}

fn print_status(rows: Vec<migration::MigrationStatusRow>) -> BackendResult<()> {
    for row in rows {
        writeln!(std::io::stdout().lock(), "{}\t{}\t{}", row.version, row.kind, row.description)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
