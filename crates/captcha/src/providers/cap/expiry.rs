use std::time::{SystemTime, UNIX_EPOCH};

use time::OffsetDateTime;
use types::http::format_utc_rfc3339_millis;

use crate::application::{CaptchaError, CaptchaResult};

use super::invalid_cap_options;

const MILLIS_PER_SECOND: i64 = 1_000;
const NANOS_PER_MILLISECOND: i128 = 1_000_000;

pub(super) struct CapExpiry {
    pub(super) epoch_millis: i64,
    pub(super) wire: String,
}

pub(super) fn expires_at(ttl_seconds: u64) -> CaptchaResult<CapExpiry> {
    let ttl_millis = i64::try_from(ttl_seconds)
        .ok()
        .and_then(|ttl| ttl.checked_mul(MILLIS_PER_SECOND))
        .ok_or_else(invalid_cap_options)?;
    let epoch_millis = now_ms()?.checked_add(ttl_millis).ok_or_else(invalid_cap_options)?;
    let wire = wire_expiry(epoch_millis)?;
    Ok(CapExpiry { epoch_millis, wire })
}

fn wire_expiry(epoch_millis: i64) -> CaptchaResult<String> {
    let timestamp = OffsetDateTime::from_unix_timestamp_nanos(i128::from(epoch_millis) * NANOS_PER_MILLISECOND).map_err(|_| invalid_cap_options())?;
    format_utc_rfc3339_millis(timestamp).map_err(|error| CaptchaError::Infrastructure(format!("CAP expiry timestamp formatting failed: {error}")))
}

fn now_ms() -> CaptchaResult<i64> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| CaptchaError::Infrastructure(format!("system clock error: {error}")))?;
    i64::try_from(elapsed.as_millis()).map_err(|error| CaptchaError::Infrastructure(format!("timestamp overflow: {error}")))
}
