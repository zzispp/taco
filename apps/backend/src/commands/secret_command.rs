use std::io::Write;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};

use crate::BackendResult;

pub(super) const JWT_SECRET_BYTES: usize = 32;

pub(super) fn generate_jwt() -> BackendResult<()> {
    let secret = generate_encoded_jwt_secret()?;
    let mut output = std::io::stdout().lock();
    write_jwt_secret(&mut output, &secret)?;
    Ok(())
}

pub(super) fn generate_encoded_jwt_secret() -> std::io::Result<String> {
    let secret = generate_random_jwt_secret()?;
    Ok(encode_jwt_secret(&secret))
}

fn generate_random_jwt_secret() -> std::io::Result<[u8; JWT_SECRET_BYTES]> {
    let mut secret = [0; JWT_SECRET_BYTES];
    OsRng
        .try_fill_bytes(&mut secret)
        .map_err(|error| std::io::Error::other(format!("operating system random source failed: {error}")))?;
    Ok(secret)
}

pub(super) fn encode_jwt_secret(secret: &[u8; JWT_SECRET_BYTES]) -> String {
    URL_SAFE_NO_PAD.encode(secret)
}

pub(super) fn write_jwt_secret(output: &mut impl Write, secret: &str) -> std::io::Result<()> {
    writeln!(output, "{secret}")
}
