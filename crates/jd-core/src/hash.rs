/// Type alias representing the 64-bit hash code used throughout the diff engine.
pub type HashCode = [u8; 8];

/// Compute the FNV-1a hash of the provided bytes.
#[must_use]
pub fn hash_bytes(input: &[u8]) -> HashCode {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash.to_le_bytes()
}

/// Combine a collection of hash codes into a single aggregate hash.
#[must_use]
pub fn combine(mut codes: Vec<HashCode>) -> HashCode {
    codes.sort_unstable();
    let mut bytes = Vec::with_capacity(codes.len() * 8);
    for code in codes {
        bytes.extend_from_slice(&code);
    }
    hash_bytes(&bytes)
}
