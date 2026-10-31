//! Horizon-only configuration. There is no foreign virtual-machine launch path in this crate.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{LauncherError, Result};

pub const GAME_VERSION: &str = "26.3";
pub const UPSTREAM_REPOSITORY: &str = "https://github.com/horizon-rust/horizon";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AndroidAbi {
    Arm64V8a,
    X86_64,
}

impl AndroidAbi {
    pub fn directory_name(self) -> &'static str {
        match self {
            Self::Arm64V8a => "arm64-v8a",
            Self::X86_64 => "x86_64",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NativePayload {
    pub game_version: String,
    pub upstream_revision: String,
    pub abi: AndroidAbi,
    pub client_library: PathBuf,
    pub resources_directory: PathBuf,
    pub sha256: String,
}

impl NativePayload {
    pub fn validate(&self, install_root: &Path) -> Result<()> {
        if self.game_version != GAME_VERSION {
            return Err(LauncherError::InvalidProfile(format!(
                "this build supports Horizon {GAME_VERSION}, not {}",
                self.game_version
            )));
        }
        for path in [&self.client_library, &self.resources_directory] {
            if !path.starts_with(install_root) {
                return Err(LauncherError::InvalidProfile(
                    "native payload path escapes the installation root".into(),
                ));
            }
        }
        Ok(())
    }
}
