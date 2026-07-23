use crate::BackendResult;

use super::migration_command::migration_command;

const CONFIG_ARGUMENT: &str = "--config";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BackendCommand {
    Serve,
    Migration(MigrationCommand),
    Administrator(AdministratorCommand),
    Secret(SecretCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MigrationCommand {
    Up,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AdministratorCommand {
    Bootstrap(AdministratorBootstrapInput),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SecretCommand {
    GenerateJwt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AdministratorBootstrapInput {
    pub username: String,
    pub email: String,
}

pub(super) fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let arguments = CommandArguments::parse(args)?;
    match command_kind(&arguments.positionals)? {
        CommandKind::Serve => arguments.into_serve(),
        CommandKind::Migration(command) => arguments.into_migration(command),
        CommandKind::AdministratorBootstrap => arguments.into_administrator_bootstrap(),
        CommandKind::GenerateJwtSecret => arguments.into_generate_jwt_secret(),
    }
}

enum CommandKind {
    Serve,
    Migration(MigrationCommand),
    AdministratorBootstrap,
    GenerateJwtSecret,
}

fn command_kind(positionals: &[String]) -> BackendResult<CommandKind> {
    match positionals {
        [] => Ok(CommandKind::Serve),
        [migration, operands @ ..] if migration == "migration" => Ok(CommandKind::Migration(migration_command(operands)?)),
        [administrator, bootstrap] if administrator == "administrator" && bootstrap == "bootstrap" => Ok(CommandKind::AdministratorBootstrap),
        [secret, generate_jwt] if secret == "secret" && generate_jwt == "generate-jwt" => Ok(CommandKind::GenerateJwtSecret),
        _ => Err(format!("unsupported taco command: {}", positionals.join(" ")).into()),
    }
}

struct CommandArguments {
    positionals: Vec<String>,
    username: Option<String>,
    email: Option<String>,
    password_stdin: bool,
    config_supplied: bool,
}

impl CommandArguments {
    fn parse(args: Vec<String>) -> BackendResult<Self> {
        let mut parsed = Self {
            positionals: Vec::new(),
            username: None,
            email: None,
            password_stdin: false,
            config_supplied: false,
        };
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                CONFIG_ARGUMENT => {
                    discard_config_path(&mut args)?;
                    parsed.config_supplied = true;
                }
                "--username" => parsed.username = set_option(parsed.username, "--username", &mut args)?,
                "--email" => parsed.email = set_option(parsed.email, "--email", &mut args)?,
                "--password-stdin" => parsed.password_stdin = set_password_stdin(parsed.password_stdin)?,
                option if option.starts_with("--") => return Err(format!("unsupported taco option: {option}").into()),
                _ => parsed.positionals.push(argument),
            }
        }
        Ok(parsed)
    }

    fn into_serve(self) -> BackendResult<BackendCommand> {
        self.reject_administrator_options("serve")?;
        Ok(BackendCommand::Serve)
    }

    fn into_migration(self, command: MigrationCommand) -> BackendResult<BackendCommand> {
        self.reject_administrator_options("migration")?;
        Ok(BackendCommand::Migration(command))
    }

    fn into_administrator_bootstrap(self) -> BackendResult<BackendCommand> {
        if !self.password_stdin {
            return Err("--password-stdin is required for administrator bootstrap".into());
        }
        Ok(BackendCommand::Administrator(AdministratorCommand::Bootstrap(AdministratorBootstrapInput {
            username: required_option("--username", self.username)?,
            email: required_option("--email", self.email)?,
        })))
    }

    fn into_generate_jwt_secret(self) -> BackendResult<BackendCommand> {
        self.reject_config("secret generate-jwt")?;
        self.reject_administrator_options("secret generate-jwt")?;
        Ok(BackendCommand::Secret(SecretCommand::GenerateJwt))
    }

    fn reject_administrator_options(&self, command: &str) -> BackendResult<()> {
        if self.username.is_none() && self.email.is_none() && !self.password_stdin {
            return Ok(());
        }
        Err(format!("{command} does not accept administrator bootstrap options").into())
    }

    fn reject_config(&self, command: &str) -> BackendResult<()> {
        if self.config_supplied {
            return Err(format!("{command} does not accept --config").into());
        }
        Ok(())
    }
}

fn discard_config_path(args: &mut std::vec::IntoIter<String>) -> BackendResult<()> {
    let value = args.next().ok_or("--config requires a file path")?;
    if value.is_empty() {
        return Err("--config requires a file path".into());
    }
    Ok(())
}

fn set_option(current: Option<String>, option: &str, args: &mut std::vec::IntoIter<String>) -> BackendResult<Option<String>> {
    if current.is_some() {
        return Err(format!("{option} may be supplied only once").into());
    }
    let value = args.next().ok_or_else(|| format!("{option} requires a value"))?;
    if value.trim().is_empty() {
        return Err(format!("{option} requires a value").into());
    }
    Ok(Some(value))
}

fn required_option(option: &str, value: Option<String>) -> BackendResult<String> {
    value.ok_or_else(|| format!("{option} is required for administrator bootstrap").into())
}

fn set_password_stdin(current: bool) -> BackendResult<bool> {
    if current {
        return Err("--password-stdin may be supplied only once".into());
    }
    Ok(true)
}
