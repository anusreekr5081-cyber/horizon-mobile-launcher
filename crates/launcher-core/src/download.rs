use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

use sha1::{Digest, Sha1};

use crate::error::{LauncherError, Result};

/// Verifies a completed download before the native game host consumes it.
pub fn verify_sha1(path: &Path, expected_lower_hex: &str) -> Result<bool> {
    if !path.is_file() {
        return Err(LauncherError::MissingFile(path.to_owned()));
    }

    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 32 * 1024];
    loop {
        let bytes = file.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }

    let actual = format!("{:x}", hasher.finalize());
    Ok(actual.eq_ignore_ascii_case(expected_lower_hex))
}

/// Atomically copies a finished HTTPS response from the Android download layer.
/// A bad or partial file never replaces an already verified target.
pub fn stage_verified_copy(
    source: &mut dyn Read,
    target: &Path,
    expected_lower_hex: &str,
) -> Result<u64> {
    let parent = target.parent().ok_or_else(|| {
        LauncherError::InvalidManifest("download target has no parent directory".into())
    })?;
    fs::create_dir_all(parent)?;

    let temporary = target.with_extension("part");
    let mut output = fs::File::create(&temporary)?;
    let mut hasher = Sha1::new();
    let mut bytes_written = 0_u64;
    let mut buffer = [0_u8; 32 * 1024];

    loop {
        let read = source.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
        bytes_written += read as u64;
    }
    output.sync_all()?;

    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected_lower_hex) {
        let _ = fs::remove_file(&temporary);
        return Err(LauncherError::ChecksumMismatch {
            path: target.to_owned(),
        });
    }

    fs::rename(temporary, target)?;
    Ok(bytes_written)
}

pub fn copy_from_reader(
    source: &mut dyn Read,
    target: &Path,
) -> io::Result<u64> {
    let parent = target.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "target has no parent")
    })?;
    fs::create_dir_all(parent)?;
    let mut output = fs::File::create(target)?;
    io::copy(source, &mut output)
}
