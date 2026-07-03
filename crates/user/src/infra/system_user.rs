use configuration::Settings;

use crate::{
    application::{SystemUserProvider, SystemUserRecord},
    domain::{User, UserId},
};

#[derive(Clone)]
pub struct ConfigSystemUserProvider {
    record: SystemUserRecord,
}

impl ConfigSystemUserProvider {
    pub fn from_settings(settings: &Settings) -> Result<Self, configuration::SettingsError> {
        Ok(Self {
            record: SystemUserRecord {
                user: User {
                    id: UserId(settings.admin.id.trim().into()),
                    username: settings.admin.username.trim().into(),
                    email: settings.admin.email.trim().into(),
                    role: settings.admin.role.trim().into(),
                    is_active: settings.admin.is_active,
                    auth_source: constants::auth::DEFAULT_AUTH_SOURCE.into(),
                    email_verified: true,
                    system: true,
                },
                password_hash: settings.admin_password_hash()?,
            },
        })
    }
}

impl SystemUserProvider for ConfigSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        Some(self.record.clone())
    }
}
