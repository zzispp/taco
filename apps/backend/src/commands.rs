use std::io::BufRead;

use configuration::Settings;
use storage::connect_database;

use crate::{BackendResult, composition, migration, startup};

mod migration_command;

use migration_command::migration_command;

pub async fn run() -> BackendResult<()> {
    let settings = Settings::load()?;
    let _tracing_guard = init_tracing(&settings)?;

    match command_from_args(std::env::args().skip(1).collect())? {
        BackendCommand::Serve => startup::serve(settings).await,
        BackendCommand::Migration(command) => run_migration(settings, command).await,
        BackendCommand::BootstrapAdmin(command) => run_bootstrap_admin(settings, command).await,
    }
}

async fn run_bootstrap_admin(settings: Settings, command: BootstrapAdminCommand) -> BackendResult<()> {
    let password = {
        let mut stdin = std::io::stdin().lock();
        read_password(&mut stdin)?
    };
    let user = composition::bootstrap_admin(
        &settings,
        user::application::BootstrapAdminInput {
            username: command.username,
            email: command.email,
            password,
        },
    )
    .await?;
    println!("bootstrap-admin created user {}", user.username);
    Ok(())
}

fn init_tracing(settings: &Settings) -> BackendResult<Option<tracing_appender::non_blocking::WorkerGuard>> {
    let config = settings.tracing_config()?;
    hook_tracing::init_global_subscriber(hook_tracing::TracingConfig {
        log_level: config.log_level,
        file_logging_enabled: config.file.enabled,
        file_directory: config.file.directory,
        file_prefix: config.file.prefix,
    })
    .map_err(Into::into)
}

async fn run_migration(settings: Settings, command: MigrationCommand) -> BackendResult<()> {
    let database = connect_database(&settings.database_url()?).await?;
    let rebuild_caches = command.rebuilds_caches();
    let pool = database.pool();
    match command {
        MigrationCommand::Up(steps) => migration::up(pool, steps).await?,
        MigrationCommand::Down(steps) => migration::down(pool, steps).await?,
        MigrationCommand::Status => print_status(migration::status(pool).await?),
        MigrationCommand::Fresh => migration::fresh(pool).await?,
        MigrationCommand::Refresh => migration::refresh(pool).await?,
        MigrationCommand::Reset => migration::reset(pool).await?,
    }
    if rebuild_caches {
        rebuild_caches_after_migration(&settings, database).await?;
    }
    Ok(())
}

fn print_status(rows: Vec<migration::MigrationStatusRow>) {
    for row in rows {
        println!("{}\t{}\t{}", row.version, row.kind, row.description);
    }
}

async fn rebuild_caches_after_migration(settings: &Settings, database: storage::Database) -> BackendResult<()> {
    composition::rebuild_rbac_cache(settings, database.clone()).await?;
    composition::rebuild_persistent_system_cache(settings, database).await
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BackendCommand {
    Serve,
    Migration(MigrationCommand),
    BootstrapAdmin(BootstrapAdminCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BootstrapAdminCommand {
    username: String,
    email: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MigrationCommand {
    Up(Option<u32>),
    Down(Option<u32>),
    Status,
    Fresh,
    Refresh,
    Reset,
}

impl MigrationCommand {
    fn rebuilds_caches(&self) -> bool {
        matches!(self, Self::Up(_) | Self::Fresh | Self::Refresh)
    }
}

fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let positionals = positional_args(args)?;
    match positionals.as_slice() {
        [] => Ok(BackendCommand::Serve),
        [migration, args @ ..] if migration == "migration" => Ok(BackendCommand::Migration(migration_command(args)?)),
        [bootstrap, args @ ..] if bootstrap == "bootstrap-admin" => Ok(BackendCommand::BootstrapAdmin(bootstrap_admin_command(args)?)),
        _ => Err(format!("unsupported backend command: {}", positionals.join(" ")).into()),
    }
}

fn bootstrap_admin_command(args: &[String]) -> BackendResult<BootstrapAdminCommand> {
    let mut username = None;
    let mut email = None;
    let mut args = args.iter();
    while let Some(option) = args.next() {
        match option.as_str() {
            "--username" => {
                let value = args.next().ok_or("--username requires a value")?;
                if username.replace(value.clone()).is_some() {
                    return Err("duplicate bootstrap-admin option: --username".into());
                }
            }
            "--email" => {
                let value = args.next().ok_or("--email requires a value")?;
                if email.replace(value.clone()).is_some() {
                    return Err("duplicate bootstrap-admin option: --email".into());
                }
            }
            _ => return Err(format!("unsupported bootstrap-admin option: {option}").into()),
        }
    }
    Ok(BootstrapAdminCommand {
        username: username.ok_or("bootstrap-admin requires --username")?,
        email: email.ok_or("bootstrap-admin requires --email")?,
    })
}

fn read_password(reader: &mut impl BufRead) -> BackendResult<String> {
    let mut password = String::new();
    if reader.read_line(&mut password)? == 0 {
        return Err("bootstrap-admin password is required on stdin".into());
    }
    while matches!(password.chars().last(), Some('\r' | '\n')) {
        password.pop();
    }
    Ok(password)
}

fn positional_args(args: Vec<String>) -> BackendResult<Vec<String>> {
    let mut positionals = Vec::new();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--config" {
            args.next().ok_or("--config requires a file path")?;
            continue;
        }
        positionals.push(arg);
    }

    Ok(positionals)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{BackendCommand, BootstrapAdminCommand, MigrationCommand, command_from_args, positional_args, read_password};

    #[test]
    fn defaults_to_serve_command() {
        assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
    }

    #[test]
    fn ignores_config_path_when_detecting_command() {
        let args = vec!["--config".into(), "config/config.local.yaml".into(), "migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up(None)));
    }

    #[test]
    fn detects_migration_up_command() {
        let args = vec!["migration".into(), "up".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Up(None)));
    }

    #[test]
    fn detects_migration_down_command() {
        let args = vec!["migration".into(), "down".into(), "2".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Down(Some(2))));
    }

    #[test]
    fn detects_migration_status_command() {
        let args = vec!["migration".into(), "status".into()];

        assert_eq!(command_from_args(args).unwrap(), BackendCommand::Migration(MigrationCommand::Status));
    }

    #[test]
    fn detects_bootstrap_admin_without_accepting_a_password_argument() {
        let args = vec![
            "bootstrap-admin".into(),
            "--username".into(),
            "root-admin".into(),
            "--email".into(),
            "root-admin@example.com".into(),
        ];

        assert_eq!(
            command_from_args(args).unwrap(),
            BackendCommand::BootstrapAdmin(BootstrapAdminCommand {
                username: "root-admin".into(),
                email: "root-admin@example.com".into(),
            })
        );
    }

    #[test]
    fn bootstrap_admin_rejects_a_password_argument() {
        let args = vec![
            "bootstrap-admin".into(),
            "--username".into(),
            "root-admin".into(),
            "--email".into(),
            "root-admin@example.com".into(),
            "--password".into(),
            "visible-secret".into(),
        ];

        assert_eq!(
            command_from_args(args).unwrap_err().to_string(),
            "unsupported bootstrap-admin option: --password"
        );
    }

    #[test]
    fn reads_bootstrap_admin_password_from_one_stdin_line() {
        let mut input = Cursor::new(b"safe-secret-123\r\nignored\n");

        assert_eq!(read_password(&mut input).unwrap(), "safe-secret-123");
    }

    #[test]
    fn rejects_missing_bootstrap_admin_password_on_stdin() {
        let mut input = Cursor::new(Vec::<u8>::new());

        assert_eq!(
            read_password(&mut input).unwrap_err().to_string(),
            "bootstrap-admin password is required on stdin"
        );
    }

    #[test]
    fn rejects_schema_commands() {
        let args = vec!["schema".into(), "push".into()];

        assert!(command_from_args(args).is_err());
    }

    #[test]
    fn rejects_missing_config_path() {
        let args = vec!["--config".into()];

        assert!(positional_args(args).is_err());
    }
}
