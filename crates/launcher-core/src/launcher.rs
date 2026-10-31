//! UI-independent launcher state. A Rust Android front end can render these
//! states with wgpu, egui, Slint, or another native Rust UI without duplicating
//! installation policy.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    error::Result,
    horizon::AndroidAbi,
    install::{HorizonInstallation, InstallStatus},
    model::Profile,
    store::LauncherStore,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LauncherSnapshot {
    pub install_status: InstallStatus,
    pub profiles: Vec<Profile>,
    pub install_root: PathBuf,
}

#[derive(Debug)]
pub struct NativeLauncher {
    profiles: LauncherStore,
    installation: HorizonInstallation,
}

impl NativeLauncher {
    pub fn open(data_root: PathBuf, abi: AndroidAbi) -> Result<Self> {
        Ok(Self {
            profiles: LauncherStore::open(data_root.clone())?,
            installation: HorizonInstallation::open(data_root, abi)?,
        })
    }

    pub fn snapshot(&self) -> Result<LauncherSnapshot> {
        Ok(LauncherSnapshot {
            install_status: self.installation.status(),
            profiles: self.profiles.list_profiles()?,
            install_root: self.installation.root().to_owned(),
        })
    }

    pub fn installation(&self) -> &HorizonInstallation {
        &self.installation
    }

    pub fn profiles(&self) -> &LauncherStore {
        &self.profiles
    }
}
