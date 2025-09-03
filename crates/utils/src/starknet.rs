//! Taken from `starknet-rs`.

use sha3::{Digest, Keccak256};
use starknet_types_core::felt::Felt;

/// Error that occurs when the string provided contains non-ASCII characters.
#[derive(Debug, thiserror::Error)]
#[error("The string provided contains non-ASCII characters.")]
pub struct NonAsciiNameError;

/// Calculates the entrypoint selector of a Starknet contract from a human-readable function name.
///
/// Returns the [Starknet Keccak](fn.starknet_keccak) of the function name in most cases, except for
/// 2 special built-in default entrypoints of `__default__` and `__l1_default__` for which `0` is
/// returned instead.
pub fn get_selector_from_name(func_name: &str) -> Result<Felt, NonAsciiNameError> {
    const DEFAULT_ENTRY_POINT_NAME: &str = "__default__";
    const DEFAULT_L1_ENTRY_POINT_NAME: &str = "__l1_default__";

    if func_name == DEFAULT_ENTRY_POINT_NAME || func_name == DEFAULT_L1_ENTRY_POINT_NAME {
        Ok(Felt::ZERO)
    } else {
        let name_bytes = func_name.as_bytes();
        if name_bytes.is_ascii() {
            Ok(starknet_keccak(name_bytes))
        } else {
            Err(NonAsciiNameError)
        }
    }
}

/// A variant of eth-keccak that computes a value that fits in a Starknet field element. It performs
/// a standard Keccak-256 but with the 6 most significant bits removed.
pub fn starknet_keccak(data: &[u8]) -> Felt {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut hash = hasher.finalize();

    // Remove the first 6 bits
    hash[0] &= 0b00000011;

    // Because we know hash is always 32 bytes
    Felt::from_bytes_be(unsafe { &*(hash[..].as_ptr() as *const [u8; 32]) })
}
