use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::keys;
use crate::{FileError, FileResult};

const DIGEST_BYTES: usize = 32;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ContentDigest([u8; DIGEST_BYTES]);

impl ContentDigest {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    pub fn from_hex(value: &str) -> FileResult<Self> {
        if value.len() != DIGEST_BYTES * 2 {
            return Err(FileError::InvalidInput(keys::DIGEST_LENGTH_INVALID));
        }
        let mut bytes = [0_u8; DIGEST_BYTES];
        for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
            bytes[index] = hex_value(chunk[0])? << 4 | hex_value(chunk[1])?;
        }
        Ok(Self(bytes))
    }

    pub const fn from_digest(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl fmt::Display for ContentDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

fn hex_value(value: u8) -> FileResult<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(FileError::InvalidInput(keys::DIGEST_FORMAT_INVALID)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_round_trips_as_lowercase_hex() {
        let digest = ContentDigest::from_bytes(b"taco");
        let parsed = ContentDigest::from_hex(&digest.to_hex()).unwrap();

        assert_eq!(parsed, digest);
        assert_eq!(digest.to_string(), digest.to_hex());
    }
}
