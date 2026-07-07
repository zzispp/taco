mod pow;

use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use constants::captcha::{CAP_CHALLENGE_COUNT, CAP_CHALLENGE_DIFFICULTY, CAP_CHALLENGE_SIZE, CAP_CHALLENGE_TTL_SECONDS, CAP_REDEEMED_TOKEN_TTL_SECONDS};
use kernel::error::LocalizedError;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::application::{CaptchaError, CaptchaProvider, CaptchaResult, CaptchaSettings};

use self::pow::{PowSolution, solution_matches};

const PROVIDER_NAME: &str = "cap";
const CHALLENGE_TOKEN_BYTES: usize = 25;
const REDEEMED_ID_BYTES: usize = 8;
const REDEEMED_SECRET_BYTES: usize = 15;
const MILLIS_PER_SECOND: i64 = 1_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapChallengeSpec {
    pub c: usize,
    pub s: usize,
    pub d: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapChallengeRecord {
    pub challenge: CapChallengeSpec,
    pub expires: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CapChallengeResponse {
    challenge: CapChallengeSpec,
    token: String,
    expires: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct CapRedeemPayload {
    token: String,
    #[serde(default)]
    solutions: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CapRedeemResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CapOptions {
    pub challenge_count: usize,
    pub challenge_size: usize,
    pub challenge_difficulty: usize,
    pub challenge_ttl_seconds: u64,
    pub redeemed_token_ttl_seconds: u64,
}

pub struct CapProvider<S> {
    store: S,
    options: CapOptions,
}

#[async_trait]
pub trait CapStore: Send + Sync + 'static {
    async fn save_challenge(&self, token: &str, record: &CapChallengeRecord, ttl_seconds: u64) -> CaptchaResult<()>;
    async fn consume_challenge(&self, token: &str) -> CaptchaResult<Option<CapChallengeRecord>>;
    async fn save_redeemed(&self, token_key: &str, expires: i64, ttl_seconds: u64) -> CaptchaResult<()>;
    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool>;
}

impl Default for CapOptions {
    fn default() -> Self {
        Self {
            challenge_count: CAP_CHALLENGE_COUNT,
            challenge_size: CAP_CHALLENGE_SIZE,
            challenge_difficulty: CAP_CHALLENGE_DIFFICULTY,
            challenge_ttl_seconds: CAP_CHALLENGE_TTL_SECONDS,
            redeemed_token_ttl_seconds: CAP_REDEEMED_TOKEN_TTL_SECONDS,
        }
    }
}

impl<S> CapProvider<S>
where
    S: CapStore,
{
    pub const fn new(store: S) -> Self {
        Self {
            store,
            options: CapOptions {
                challenge_count: CAP_CHALLENGE_COUNT,
                challenge_size: CAP_CHALLENGE_SIZE,
                challenge_difficulty: CAP_CHALLENGE_DIFFICULTY,
                challenge_ttl_seconds: CAP_CHALLENGE_TTL_SECONDS,
                redeemed_token_ttl_seconds: CAP_REDEEMED_TOKEN_TTL_SECONDS,
            },
        }
    }

    pub const fn with_options(store: S, options: CapOptions) -> Self {
        Self { store, options }
    }

    async fn redeem_cap_payload(&self, settings: &CaptchaSettings, payload: CapRedeemPayload) -> CaptchaResult<CapRedeemResponse> {
        let Some(record) = self.store.consume_challenge(&payload.token).await? else {
            return Ok(CapRedeemResponse::failure("invalid_or_expired_challenge"));
        };
        if !solutions_match(&payload.token, &record.challenge, &payload.solutions) {
            return Ok(CapRedeemResponse::failure("invalid_solution"));
        }
        self.save_redeemed_token(&self.options(settings)?).await
    }

    async fn save_redeemed_token(&self, options: &CapOptions) -> CaptchaResult<CapRedeemResponse> {
        let token = redeemed_token();
        let key = redeemed_token_key(&token)?;
        let expires = expires_at(options.redeemed_token_ttl_seconds);
        self.store.save_redeemed(&key, expires, options.redeemed_token_ttl_seconds).await?;
        Ok(CapRedeemResponse::success(token, expires))
    }

    fn options(&self, settings: &CaptchaSettings) -> CaptchaResult<CapOptions> {
        let config = settings.provider_config(PROVIDER_NAME)?;
        if config.as_object().is_some_and(serde_json::Map::is_empty) {
            return Ok(self.options.clone());
        }
        serde_json::from_value(config.clone()).map_err(invalid_cap_config)
    }
}

#[async_trait]
impl<S> CaptchaProvider for CapProvider<S>
where
    S: CapStore,
{
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn public_config(&self, settings: &CaptchaSettings) -> CaptchaResult<Value> {
        Ok(settings.provider_config(PROVIDER_NAME)?.clone())
    }

    async fn challenge(&self, settings: &CaptchaSettings) -> CaptchaResult<Value> {
        let options = self.options(settings)?;
        let challenge = challenge(&options);
        let token = random_hex(CHALLENGE_TOKEN_BYTES);
        let expires = expires_at(options.challenge_ttl_seconds);
        let record = CapChallengeRecord {
            challenge: challenge.clone(),
            expires,
        };
        self.store.save_challenge(&token, &record, options.challenge_ttl_seconds).await?;
        to_value(CapChallengeResponse { challenge, token, expires })
    }

    async fn redeem(&self, settings: &CaptchaSettings, payload: Value) -> CaptchaResult<Value> {
        let payload = serde_json::from_value(payload).map_err(invalid_redeem_payload)?;
        to_value(self.redeem_cap_payload(settings, payload).await?)
    }

    async fn verify(&self, _settings: &CaptchaSettings, token: Option<&str>) -> CaptchaResult<()> {
        let token = token.filter(|value| !value.trim().is_empty()).ok_or_else(required_error)?;
        let key = redeemed_token_key(token)?;
        if self.store.consume_redeemed(&key).await? {
            return Ok(());
        }
        Err(CaptchaError::InvalidInput(localized("errors.captcha.verification_failed")))
    }
}

impl CapRedeemResponse {
    fn success(token: String, expires: i64) -> Self {
        Self {
            success: true,
            token: Some(token),
            expires: Some(expires),
            reason: None,
            error: None,
        }
    }

    fn failure(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        Self {
            success: false,
            token: None,
            expires: None,
            reason: Some(reason.clone()),
            error: Some(reason),
        }
    }
}

fn challenge(options: &CapOptions) -> CapChallengeSpec {
    CapChallengeSpec {
        c: options.challenge_count,
        s: options.challenge_size,
        d: options.challenge_difficulty,
    }
}

fn solutions_match(token: &str, challenge: &CapChallengeSpec, solutions: &[u64]) -> bool {
    solutions.len() == challenge.c
        && solutions.iter().enumerate().all(|(index, solution)| {
            solution_matches(PowSolution {
                token,
                index: index + 1,
                spec: challenge,
                solution: *solution,
            })
        })
}

#[cfg(test)]
pub(crate) fn solve_for_test(token: &str, spec: &CapChallengeSpec, index: usize) -> u64 {
    (0..u64::MAX)
        .find(|candidate| {
            solution_matches(PowSolution {
                token,
                index,
                spec,
                solution: *candidate,
            })
        })
        .expect("test challenge must be solvable")
}

fn redeemed_token() -> String {
    format!("{}:{}", random_hex(REDEEMED_ID_BYTES), random_hex(REDEEMED_SECRET_BYTES))
}

fn redeemed_token_key(token: &str) -> CaptchaResult<String> {
    let Some((id, secret)) = token.split_once(':') else {
        return Err(CaptchaError::InvalidInput(localized("errors.captcha.token_invalid")));
    };
    if id.is_empty() || secret.is_empty() {
        return Err(CaptchaError::InvalidInput(localized("errors.captcha.token_invalid")));
    }
    let hash = Sha256::digest(secret.as_bytes());
    Ok(format!("{id}:{}", hex::encode(hash)))
}

fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0_u8; bytes];
    OsRng.fill_bytes(&mut buffer);
    hex::encode(buffer)
}

fn expires_at(ttl_seconds: u64) -> i64 {
    now_ms() + i64::try_from(ttl_seconds).expect("captcha TTL seconds must fit i64") * MILLIS_PER_SECOND
}

fn now_ms() -> i64 {
    let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock must be after Unix epoch");
    i64::try_from(elapsed.as_millis()).expect("current timestamp must fit i64")
}

fn required_error() -> CaptchaError {
    CaptchaError::InvalidInput(localized("errors.captcha.verification_required"))
}

fn invalid_redeem_payload(error: serde_json::Error) -> CaptchaError {
    let _ = error;
    CaptchaError::InvalidInput(localized("errors.captcha.invalid_redeem_payload"))
}

fn invalid_cap_config(error: serde_json::Error) -> CaptchaError {
    let _ = error;
    CaptchaError::InvalidInput(localized("errors.captcha.invalid_provider_config"))
}

fn to_value<T: Serialize>(value: T) -> CaptchaResult<Value> {
    serde_json::to_value(value).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> CaptchaError {
    CaptchaError::Infrastructure(format!("captcha json error: {error}"))
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}
