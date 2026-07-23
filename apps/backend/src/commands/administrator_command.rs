use std::io::{BufRead, Error, ErrorKind, Write};

use configuration::Settings;
use storage::connect_database;
use user::application::{BootstrapAdministratorInput, BootstrapAdministratorOutcome};

use super::parser::AdministratorBootstrapInput;
use crate::{BackendResult, composition, migration, startup};

pub(super) async fn bootstrap(settings: Settings, input: AdministratorBootstrapInput) -> BackendResult<()> {
    startup::prepare_runtime_schema(&settings).await?;
    let database = connect_database(&settings.database_url()?).await?;
    migration::ensure_runtime_schema_ready(database.raw_pool()).await?;
    let password = read_password_from_standard_input().await?;
    let outcome = composition::bootstrap_administrator(
        database,
        BootstrapAdministratorInput {
            username: input.username,
            email: input.email,
            password,
        },
    )
    .await?;
    report_bootstrap_outcome(outcome)
}

async fn read_password_from_standard_input() -> BackendResult<String> {
    tokio::task::spawn_blocking(|| {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        password_from_reader(&mut reader)
    })
    .await
    .map_err(|error| Error::other(format!("administrator password input task failed: {error}")))?
    .map_err(Into::into)
}

fn password_from_reader(reader: &mut impl BufRead) -> Result<String, Error> {
    let mut password = String::new();
    if reader.read_line(&mut password)? == 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "administrator password input is empty"));
    }
    remove_line_ending(&mut password);
    if password.trim().is_empty() {
        return Err(Error::new(ErrorKind::InvalidInput, "administrator password input is empty"));
    }
    Ok(password)
}

fn remove_line_ending(value: &mut String) {
    if value.ends_with('\n') {
        value.pop();
    }
    if value.ends_with('\r') {
        value.pop();
    }
}

fn report_bootstrap_outcome(outcome: BootstrapAdministratorOutcome) -> BackendResult<()> {
    match outcome {
        BootstrapAdministratorOutcome::Created => {
            writeln!(std::io::stdout().lock(), "administrator bootstrap completed")?;
            Ok(())
        }
        BootstrapAdministratorOutcome::AlreadyPresent => Err("an enabled system administrator already exists; bootstrap did not create a user".into()),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::password_from_reader;

    #[test]
    fn reads_one_password_line_without_the_terminal_line_ending() {
        let mut reader = Cursor::new("correct-horse-battery-staple\r\nignored");

        let password = password_from_reader(&mut reader).unwrap();

        assert_eq!(password, "correct-horse-battery-staple");
    }

    #[test]
    fn rejects_empty_password_input() {
        for contents in ["", "\n", "   \r\n"] {
            let mut reader = Cursor::new(contents);
            assert!(password_from_reader(&mut reader).is_err());
        }
    }
}
