use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};

use super::{JwtSecretGenerator, SetupPortFailure};

const JWT_SECRET_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
pub struct RandomJwtSecretGenerator;

impl JwtSecretGenerator for RandomJwtSecretGenerator {
    fn generate_jwt_secret(&self) -> Result<String, SetupPortFailure> {
        let mut bytes = [0; JWT_SECRET_BYTES];
        OsRng.fill_bytes(&mut bytes);
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }
}

pub(super) fn valid_jwt_secret(secret: &str) -> bool {
    secret.trim().len() >= JWT_SECRET_BYTES
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    use super::{JWT_SECRET_BYTES, JwtSecretGenerator, RandomJwtSecretGenerator, valid_jwt_secret};

    #[test]
    fn random_generator_emits_a_base64url_32_byte_jwt_secret() {
        let secret = RandomJwtSecretGenerator.generate_jwt_secret().unwrap();
        let decoded = URL_SAFE_NO_PAD.decode(&secret).unwrap();

        assert_eq!(decoded.len(), JWT_SECRET_BYTES);
        assert!(valid_jwt_secret(&secret));
    }
}
