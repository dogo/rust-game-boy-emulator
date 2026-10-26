// Biblioteca pública do emulador Game Boy
// Permite que integration tests e outros crates usem o código

pub mod GB;

use std::hash::{BuildHasher, RandomState};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

static RANDOM_STATE: OnceLock<AtomicU64> = OnceLock::new();

pub fn random_u64() -> u64 {
    let state =
        RANDOM_STATE.get_or_init(|| AtomicU64::new(RandomState::new().hash_one(0u8).max(1)));
    state
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |mut value| {
            value ^= value << 13;
            value ^= value >> 7;
            value ^= value << 17;
            Some(value)
        })
        .unwrap()
}

pub(crate) fn fill_random(bytes: &mut [u8]) {
    for chunk in bytes.chunks_mut(8) {
        chunk.copy_from_slice(&random_u64().to_ne_bytes()[..chunk.len()]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn random_words_advance() {
        assert_ne!(super::random_u64(), super::random_u64());
    }
}
