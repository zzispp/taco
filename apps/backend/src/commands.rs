use std::io::Write;

use configuration::Settings;
use storage::connect_database;

use crate::{BackendResult, migration, startup};

mod administrator_command;
mod migration_command;
mod parser;
mod secret_command;

use parser::{AdministratorCommand, BackendCommand, MigrationCommand, SecretCommand, command_from_args};

pub async fn run() -> BackendResult<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let command = command_from_args(args.clone())?;
    match command {
        BackendCommand::Secret(SecretCommand::GenerateJwt) => secret_command::generate_jwt(),
        BackendCommand::Serve => startup::serve(Settings::load_from_args(args)?).await,
        BackendCommand::Migration(command) => run_migration(Settings::load_from_args(args)?, command).await,
        BackendCommand::Administrator(command) => run_administrator(Settings::load_from_args(args)?, command).await,
    }
}

async fn run_administrator(settings: Settings, command: AdministratorCommand) -> BackendResult<()> {
    match command {
        AdministratorCommand::Bootstrap(input) => administrator_command::bootstrap(settings, input).await,
    }
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
