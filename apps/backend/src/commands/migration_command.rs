use crate::BackendResult;

use super::parser::MigrationCommand;

pub(super) fn migration_command(args: &[String]) -> BackendResult<MigrationCommand> {
    match args {
        [command] if command == "up" => Ok(MigrationCommand::Up),
        [command] if command == "status" => Ok(MigrationCommand::Status),
        _ => Err(format!("unsupported migration command: {}", args.join(" ")).into()),
    }
}
