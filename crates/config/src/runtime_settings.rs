use sha2::{Digest, Sha256};

use crate::{Settings, SettingsError, settings::required_config_value};

const KNOWN_INSECURE_JWT_SECRET_SHA256: [[u8; 32]; 2] = [
    [
        0xb8, 0x9f, 0x85, 0xb2, 0x25, 0x06, 0xeb, 0x72, 0xf6, 0x0b, 0x3a, 0x7b, 0x3c, 0xa1, 0xd0, 0x9b, 0x74, 0x90, 0xd0, 0xe0, 0x52, 0xe2, 0x08, 0xec, 0xfd,
        0xa4, 0x88, 0xe0, 0x7a, 0x09, 0x66, 0x8f,
    ],
    [
        0x33, 0xba, 0xdc, 0x0f, 0x05, 0x15, 0xad, 0xd7, 0x09, 0xe4, 0x25, 0xf5, 0x94, 0x57, 0xec, 0x0f, 0x96, 0x61, 0xf5, 0x65, 0x7b, 0xca, 0x54, 0xc7, 0x58,
        0x48, 0x78, 0x1c, 0x1d, 0x48, 0x77, 0x8f,
    ],
];
const MIN_JWT_SECRET_BYTES: usize = 32;
impl Settings {
    pub fn bind_addr(&self) -> String {
        match self.server.host.contains(':') {
            true => format!("[{}]:{}", self.server.host, self.server.port),
            false => format!("{}:{}", self.server.host, self.server.port),
        }
    }

    pub fn jwt_secret(&self) -> Result<String, SettingsError> {
        let secret = required_config_value("jwt.secret", &self.jwt.secret)?;
        if is_known_insecure_jwt_secret(&secret) {
            return Err(SettingsError::InsecureJwtSecret);
        }
        let actual_bytes = secret.len();
        if actual_bytes < MIN_JWT_SECRET_BYTES {
            return Err(SettingsError::JwtSecretTooShort {
                minimum_bytes: MIN_JWT_SECRET_BYTES,
                actual_bytes,
            });
        }
        Ok(secret)
    }
}

fn is_known_insecure_jwt_secret(secret: &str) -> bool {
    let digest: [u8; 32] = Sha256::digest(secret.as_bytes()).into();
    KNOWN_INSECURE_JWT_SECRET_SHA256.contains(&digest)
}
