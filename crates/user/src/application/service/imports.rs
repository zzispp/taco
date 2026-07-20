use super::*;

pub(super) struct PreparedUserImport {
    pub(super) writes: Vec<UserImportWrite>,
    pub(super) report: UserImportReport,
}

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    pub(super) async fn prepare_user_import(&self, input: UserImportInput) -> AppResult<PreparedUserImport> {
        if input.rows.is_empty() {
            return Err(AppError::InvalidInput(localized("errors.user.import_empty")));
        }

        let mut writes = Vec::with_capacity(input.rows.len());
        let mut messages = Vec::with_capacity(input.rows.len());
        let mut failures = Vec::new();
        for row in input.rows {
            match self.prepare_import_row(row, input.update_support).await {
                Ok((write, message)) => {
                    writes.push(write);
                    messages.push(message);
                }
                Err(error) => collect_import_failure(&mut failures, error)?,
            }
        }
        if !failures.is_empty() {
            return Err(AppError::ImportValidation(failures));
        }
        Ok(PreparedUserImport {
            writes,
            report: UserImportReport {
                success_count: messages.len(),
                messages,
            },
        })
    }

    async fn prepare_import_row(&self, row: UserImportRow, update_support: bool) -> AppResult<(UserImportWrite, UserImportMessage)> {
        let username = row.username.trim().to_owned();
        let found = self.repository.find_auth_by_username(&username).await?;
        match found {
            None => self.prepare_imported_user(row).await,
            Some(existing) if update_support => self.prepare_import_update(row, existing.user).await,
            Some(_) => Err(AppError::Conflict(localized_param("errors.user.import_account_exists", "username", username))),
        }
    }

    async fn prepare_imported_user(&self, row: UserImportRow) -> AppResult<(UserImportWrite, UserImportMessage)> {
        let username = row.username.trim().to_owned();
        let record = self
            .prepare_new_user(NewUser {
                username: username.clone(),
                password: row.password,
                nick_name: row.nick_name,
                dept_id: row.dept_id,
                email: row.email,
                phonenumber: row.phonenumber,
                sex: default_if_blank(row.sex, "2"),
                status: default_if_blank(row.status, "0"),
                remark: None,
                role_ids: Vec::new(),
                post_ids: vec![],
            })
            .await?;
        Ok((UserImportWrite::Create(record), UserImportMessage::new(IMPORT_ACCOUNT_CREATED_KEY, username)))
    }

    async fn prepare_import_update(&self, row: UserImportRow, existing: User) -> AppResult<(UserImportWrite, UserImportMessage)> {
        self.reject_installation_owner_mutation(&existing.id).await?;
        let username = row.username.trim().to_owned();
        let record = self
            .prepare_replacement(
                &existing.id,
                ReplaceUser {
                    username: username.clone(),
                    password: Some(row.password),
                    nick_name: row.nick_name,
                    dept_id: existing.dept_id,
                    email: row.email,
                    phonenumber: row.phonenumber,
                    sex: default_if_blank(row.sex, "2"),
                    status: default_if_blank(row.status, "0"),
                    remark: existing.remark,
                    role_ids: existing.role_ids,
                    post_ids: existing.post_ids,
                },
            )
            .await?;
        Ok((
            UserImportWrite::Replace { id: existing.id, user: record },
            UserImportMessage::new(IMPORT_ACCOUNT_UPDATED_KEY, username),
        ))
    }
}

fn collect_import_failure(failures: &mut Vec<LocalizedError>, error: AppError) -> AppResult<()> {
    match error {
        AppError::InvalidInput(error) | AppError::Conflict(error) => {
            failures.push(error);
            Ok(())
        }
        error => Err(error),
    }
}
