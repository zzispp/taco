use sha2::{Digest, Sha256};

use super::CapChallengeSpec;

const FNV_OFFSET: u32 = 2_166_136_261;

pub(crate) fn solution_matches(token: &str, index: usize, spec: &CapChallengeSpec, solution: u64) -> bool {
    let salt = prng(&format!("{token}{index}"), spec.s);
    let target = prng(&format!("{token}{index}d"), spec.d);
    let digest = Sha256::digest(format!("{salt}{solution}").as_bytes());
    hex::encode(digest).starts_with(&target)
}

fn prng(seed: &str, length: usize) -> String {
    let mut state = fnv1a(seed);
    let mut result = String::new();
    while result.len() < length {
        state = next_state(state);
        result.push_str(&format!("{state:08x}"));
    }
    result.truncate(length);
    result
}

fn fnv1a(value: &str) -> u32 {
    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash
            .wrapping_add(hash << 1)
            .wrapping_add(hash << 4)
            .wrapping_add(hash << 7)
            .wrapping_add(hash << 8)
            .wrapping_add(hash << 24);
    }
    hash
}

fn next_state(mut state: u32) -> u32 {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state
}
