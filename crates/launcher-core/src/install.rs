//! Native, Rust-only installation state for Horizon 26.3 on Android.
//! Files arrive through a verified Rust downloader; this module never invokes
//! a foreign virtual machine, a shell, or an arbitrary executable.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{LauncherError, Result},
    horizon::{AndroidAbi, NativePayload, GAME_VERSION},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InstallStatus {
    Missing,
    Downloading,
    Verifying,
    Ready,
    NeedsRepair { reason: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstallRecord {
    pub schema: u32,
    pub payload: NativePayload,
}

#[derive(Debug)]
pub struct HorizonInstallation {
    root: PathBuf,
    abi: AndroidAbi,
}

impl HorizonInstallation {
    pub fn open(root: impl Into<PathBuf>, abi: AndroidAbi) -> Result<Self> {
        let root = root.into().join("horizon").join(GAME_VERSION);
        fs::create_dir_all(root.join("payload"))?;
        fs::create_dir_all(root.join("resources"))?;
        Ok(Self { root, abi })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn status(&self) -> InstallStatus {
        match self.load_record() {
            Err(error) => InstallStatus::NeedsRepair {
                reason: error.to_string(),
            },
            Ok(None) => InstallStatus::Missing,
            Ok(Some(record)) => match self.validate_record(&record) {
                Ok(()) => InstallStatus::Ready,
                Err(error) => InstallStatus::NeedsRepair {
                    reason: error.to_string(),
                },
            },
        }
    }

    pub fn register_verified_payload(&self, payload: NativePayload) -> Result<()> {
        self.validate_payload(&payload)?;
        let data = serde_json::to_vec_pretty(&InstallRecord { schema: 1, payload })?;
        atomic_write(&self.record_path(), &data)
    }

    pub fn native_library(&self) -> Result<PathBuf> {
        let record = self
            .load_record()?
            .ok_or_else(|| LauncherError::MissingFile(self.record_path()))?;
        self.validate_record(&record)?;
        Ok(record.payload.client_library)
    }

    fn record_path(&self) -> PathBuf {
        self.root.join("install.json")
    }

    fn load_record(&self) -> Result<Option<InstallRecord>> {
        let path = self.record_path();
        if !path.exists() {
            return Ok(None);
        }
        let record: InstallRecord = serde_json::from_slice(&fs::read(path)?)?;
        if record.schema != 1 {
            return Err(LauncherError::InvalidProfile(
                "unsupported installation record schema".into(),
            ));
        }
        Ok(Some(record))
    }

    fn validate_record(&self, record: &InstallRecord) -> Result<()> {
        self.validate_payload(&record.payload)?;
        if !record.payload.client_library.is_file() {
            return Err(LauncherError::MissingFile(
                record.payload.client_library.clone(),
            ));
        }
        if !record.payload.resources_directory.is_dir() {
            return Err(LauncherError::MissingFile(
                record.payload.resources_directory.clone(),
            ));
        }
        Ok(())
    }

    fn validate_payload(&self, payload: &NativePayload) -> Result<()> {
        if payload.abi != self.abi {
            return Err(LauncherError::InvalidProfile(format!(
                "payload ABI {} does not match this device's {}",
                payload.abi.directory_name(),
                self.abi.directory_name()
            )));
        }
        payload.validate(&self.root)
    }
}

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        LauncherError::InvalidProfile("installation record has no parent directory".into())
    })?;
    fs::create_dir_all(parent)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_extension(format!("tmp.{nonce}"));
    {
        let mut file = fs::File::create(&temporary)?;
        file.write_all(data)?;
        file.sync_all()?;
    }
    if let Err(err) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(err.into());
    }
    Ok(())
}
