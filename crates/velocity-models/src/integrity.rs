use ring::digest::{Context, SHA256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use velocity_core::{VelocityError, VelocityResult};

/// Computes the SHA-256 checksum of a file in 64KB streamed chunks.
pub fn compute_sha256(path: &Path) -> VelocityResult<String> {
    let file = File::open(path)
        .map_err(|e| VelocityError::Model(format!("Failed to open file for checksum {}: {}", path.display(), e)))?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|e| VelocityError::Model(format!("Failed reading file for checksum: {}", e)))?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    let digest = context.finish();
    let hex_str: String = digest
        .as_ref()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    Ok(hex_str)
}

/// Verifies whether a file matches the expected SHA-256 hexadecimal string.
pub fn verify_sha256(path: &Path, expected_hex: &str) -> VelocityResult<bool> {
    let actual_hex = compute_sha256(path)?;
    Ok(actual_hex.eq_ignore_ascii_case(expected_hex.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compute_sha256() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("velocity_sha256_test.txt");
        let mut f = File::create(&test_file).unwrap();
        f.write_all(b"VELOCITY AI").unwrap();
        drop(f);

        // SHA-256 of "VELOCITY AI" is:
        // python -c "import hashlib; print(hashlib.sha256(b'VELOCITY AI').hexdigest())"
        // -> 16fc15ba9253aa49c6e91ad853afe8351b14e5aa2fce1e59aad10d3ed5141f26
        let hash = compute_sha256(&test_file).unwrap();
        assert_eq!(
            hash,
            "16fc15ba9253aa49c6e91ad853afe8351b14e5aa2fce1e59aad10d3ed5141f26"
        );

        assert!(verify_sha256(&test_file, &hash).unwrap());
        let _ = std::fs::remove_file(test_file);
    }
}

