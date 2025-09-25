/// Type alias representing the 64-bit hash code used throughout the diff engine.
///
/// ```
/// # use jd_core::hash_bytes;
/// let code = hash_bytes(b"jd");
/// assert_eq!(code.len(), 8);
/// ```
pub type HashCode = [u8; 8];

/// Compute the FNV-1a hash of the provided bytes.
///
/// ```
/// # use jd_core::hash_bytes;
/// let code = hash_bytes(b"diff");
/// let same = hash_bytes(b"diff");
/// assert_eq!(code, same);
/// ```
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
///
/// ```
/// # use jd_core::{combine, hash_bytes};
/// let hashes = vec![hash_bytes(b"a"), hash_bytes(b"b")];
/// let combined = combine(hashes);
/// assert_eq!(combined.len(), 8);
/// ```
#[must_use]
pub fn combine(mut codes: Vec<HashCode>) -> HashCode {
    codes.sort_unstable();
    let mut bytes = Vec::with_capacity(codes.len() * 8);
    for code in codes {
        bytes.extend_from_slice(&code);
    }
    hash_bytes(&bytes)
}
