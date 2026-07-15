use crate::BackendResult;

use super::MigrationCommand;

pub(super) fn migration_command(args: &[String]) -> BackendResult<MigrationCommand> {
    let Some((command, operands)) = args.split_first() else {
        return Ok(MigrationCommand::Up(None));
    };
    let parsed = match command.as_str() {
        "up" => stepped_migration(operands, None, MigrationCommand::Up)?,
        "down" => stepped_migration(operands, Some(1), MigrationCommand::Down)?,
        "status" => fixed_migration(operands, MigrationCommand::Status),
        "fresh" => fixed_migration(operands, MigrationCommand::Fresh),
        "refresh" => fixed_migration(operands, MigrationCommand::Refresh),
        "reset" => fixed_migration(operands, MigrationCommand::Reset),
        _ => None,
    };
    parsed.ok_or_else(|| format!("unsupported migration command: {}", args.join(" ")).into())
}

fn stepped_migration(
    operands: &[String],
    default_steps: Option<u32>,
    build: impl Fn(Option<u32>) -> MigrationCommand,
) -> BackendResult<Option<MigrationCommand>> {
    match operands {
        [] => Ok(Some(build(default_steps))),
        [steps] => Ok(Some(build(Some(parse_steps(steps)?)))),
        _ => Ok(None),
    }
}

fn fixed_migration(operands: &[String], command: MigrationCommand) -> Option<MigrationCommand> {
    operands.is_empty().then_some(command)
}

fn parse_steps(value: &str) -> BackendResult<u32> {
    value
        .parse::<u32>()
        .map_err(|error| format!("invalid migration step count '{value}': {error}").into())
}
