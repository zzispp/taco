use crate::{HttpSettings, Settings, SettingsError};

impl Settings {
    pub fn http_config(&self) -> Result<HttpSettings, SettingsError> {
        if self.http.request_timeout_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("http.request_timeout_ms"));
        }
        Ok(self.http.clone())
    }

    pub fn scheduler_config(&self) -> Result<crate::SchedulerSettings, SettingsError> {
        if self.scheduler.http_client.request_timeout_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("scheduler.http_client.request_timeout_ms"));
        }
        if self.scheduler.runtime.reconcile_interval_ms == 0 {
            return Err(SettingsError::NonPositiveNumber("scheduler.runtime.reconcile_interval_ms"));
        }
        Ok(self.scheduler.clone())
    }

    pub fn audit_config(&self) -> Result<crate::AuditSettings, SettingsError> {
        positive("audit.outbox.worker_count", self.audit.outbox.worker_count)?;
        positive("audit.outbox.claim_batch_size", self.audit.outbox.claim_batch_size)?;
        positive("audit.outbox.poll_interval_ms", self.audit.outbox.poll_interval_ms)?;
        positive("audit.outbox.lease_duration_ms", self.audit.outbox.lease_duration_ms)?;
        positive("audit.outbox.retry_delay_ms", self.audit.outbox.retry_delay_ms)?;
        positive("audit.outbox.cleanup_interval_ms", self.audit.outbox.cleanup_interval_ms)?;
        positive("audit.outbox.cleanup_batch_size", self.audit.outbox.cleanup_batch_size)?;
        positive("audit.outbox.processed_retention_days", self.audit.outbox.processed_retention_days)?;
        Ok(self.audit.clone())
    }

    pub fn online_session_config(&self) -> Result<crate::OnlineSessionSettings, SettingsError> {
        positive("user.online_sessions.cleanup_interval_ms", self.user.online_sessions.cleanup_interval_ms)?;
        positive("user.online_sessions.cleanup_batch_size", self.user.online_sessions.cleanup_batch_size)?;
        Ok(self.user.online_sessions.clone())
    }

    pub fn client_info_config(&self) -> Result<crate::ClientInfoSettings, SettingsError> {
        positive("client_info.ip_location.request_timeout_ms", self.client_info.ip_location.request_timeout_ms)?;
        Ok(self.client_info.clone())
    }

    pub(crate) fn validate(&self) -> Result<(), SettingsError> {
        validate_data_directory(&self.data_directory)?;
        crate::settings::required_config_value("server.host", &self.server.host)?;
        positive("server.port", self.server.port)?;
        self.database_url()?;
        self.redis_url()?;
        self.jwt_secret()?;
        self.http_config()?;
        self.scheduler_config()?;
        self.audit_config()?;
        self.online_session_config()?;
        self.client_info_config()?;
        Ok(())
    }
}

fn validate_data_directory(path: &std::path::Path) -> Result<(), SettingsError> {
    validate_data_directory_value(path)?;
    if !path.is_absolute() {
        return Err(SettingsError::RelativeDataDirectory);
    }
    Ok(())
}

pub(crate) fn validate_data_directory_value(path: &std::path::Path) -> Result<(), SettingsError> {
    if path.as_os_str().is_empty() {
        return Err(SettingsError::BlankConfigValue("data_directory"));
    }
    let value = path.to_string_lossy();
    crate::settings::validated_config_value("data_directory", &value)?;
    Ok(())
}

fn positive(key: &'static str, value: impl PartialEq + From<u8>) -> Result<(), SettingsError> {
    if value == 0_u8.into() {
        return Err(SettingsError::NonPositiveNumber(key));
    }
    Ok(())
}
