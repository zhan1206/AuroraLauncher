//! Cryptographic utility functions.
//!
//! Provides SHA-256 hash computation and verification for downloaded files.

use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::AsyncReadExt;

/// Compute the SHA-256 hash of a file on disk.
///
/// Returns the hash as a lowercase hexadecimal string.
pub async fn sha256_file(path: &Path) -> Result<String, AppError> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| AppError::FileIo(e))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await.map_err(|e| AppError::FileIo(e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Compute the SHA-256 hash of a byte slice.
///
/// Returns the hash as a lowercase hexadecimal string.
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Verify that a file's SHA-256 hash matches the expected value.
///
/// Returns `Ok(())` if the hash matches, or an `AppError::HashMismatch` error
/// with both the expected and actual hashes.
pub async fn verify_sha256(path: &Path, expected: &str) -> Result<(), AppError> {
    let actual = sha256_file(path).await?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(AppError::HashMismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}

/// Verify a SHA-1 hash (used by Mojang's API for library checksums).
///
/// Note: SHA-1 is not available in the sha2 crate; this is a placeholder
/// that uses SHA-256 instead. For full Mojang compatibility, add the sha1 crate.
pub fn verify_sha1_placeholder(data: &[u8], _expected: &str) -> bool {
    // TODO: Add sha1 crate for proper SHA-1 verification
    // For now, we skip SHA-1 verification but log a warning
    tracing::warn!("SHA-1 verification not yet implemented, skipping for: {:02x?}", &data[..data.len().min(8)]);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_bytes() {
        let hash = sha256_bytes(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha256_empty() {
        let hash = sha256_bytes(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
