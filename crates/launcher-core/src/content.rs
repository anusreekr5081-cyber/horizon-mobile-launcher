//! Content and Add-on manager for Horizon 26.3 on Android.
//! Handles isolated instance worlds, mods, and resource packs without
//! foreign VM code or elevated root permissions.

use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

use crate::error::{LauncherError, Result};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorldInfo {
    pub folder_name: String,
    pub display_name: String,
    pub size_bytes: u64,
    pub last_modified_unix: u64,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ModInfo {
    pub filename: String,
    pub enabled: bool,
    pub size_bytes: u64,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourcePackInfo {
    pub filename: String,
    pub enabled: bool,
    pub size_bytes: u64,
    pub path: PathBuf,
}

/// Lists all save worlds in the given instance game directory (`<instance>/saves`).
pub fn list_worlds(game_dir: &Path) -> Result<Vec<WorldInfo>> {
    let saves_dir = game_dir.join("saves");
    if !saves_dir.exists() {
        return Ok(Vec::new());
    }

    let mut worlds = Vec::new();
    for entry in fs::read_dir(&saves_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let folder_name = entry.file_name().to_string_lossy().into_owned();
            let metadata = entry.metadata()?;
            let last_modified_unix = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let size_bytes = calculate_directory_size(&path);
            let display_name = folder_name.replace('_', " ");

            worlds.push(WorldInfo {
                folder_name,
                display_name,
                size_bytes,
                last_modified_unix,
                path,
            });
        }
    }
    worlds.sort_by(|a, b| b.last_modified_unix.cmp(&a.last_modified_unix));
    Ok(worlds)
}

/// Deletes a world safely from the instance.
pub fn delete_world(game_dir: &Path, folder_name: &str) -> Result<()> {
    validate_filename(folder_name)?;
    let world_dir = game_dir.join("saves").join(folder_name);
    if !world_dir.exists() {
        return Err(LauncherError::MissingFile(world_dir));
    }
    fs::remove_dir_all(world_dir)?;
    Ok(())
}

/// Lists all mods installed in the instance `mods` directory.
/// Files ending in `.disabled` are reported as `enabled: false`.
pub fn list_mods(game_dir: &Path) -> Result<Vec<ModInfo>> {
    let mods_dir = game_dir.join("mods");
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }

    let mut mods = Vec::new();
    for entry in fs::read_dir(&mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let filename = entry.file_name().to_string_lossy().into_owned();
            let is_disabled = filename.ends_with(".disabled");
            let size_bytes = entry.metadata()?.len();

            mods.push(ModInfo {
                filename,
                enabled: !is_disabled,
                size_bytes,
                path,
            });
        }
    }
    mods.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(mods)
}

/// Toggles a mod between active (.jar / .zip / .so) and inactive (.disabled).
pub fn toggle_mod(game_dir: &Path, filename: &str, enable: bool) -> Result<PathBuf> {
    validate_filename(filename)?;
    let mods_dir = game_dir.join("mods");
    let current_path = mods_dir.join(filename);
    if !current_path.is_file() {
        return Err(LauncherError::MissingFile(current_path));
    }

    let new_filename = if enable {
        if let Some(stripped) = filename.strip_suffix(".disabled") {
            stripped.to_string()
        } else {
            filename.to_string()
        }
    } else if !filename.ends_with(".disabled") {
        format!("{filename}.disabled")
    } else {
        filename.to_string()
    };

    let new_path = mods_dir.join(&new_filename);
    if current_path != new_path {
        fs::rename(&current_path, &new_path)?;
    }
    Ok(new_path)
}

/// Lists all resource packs in the instance `resourcepacks` directory.
pub fn list_resource_packs(game_dir: &Path) -> Result<Vec<ResourcePackInfo>> {
    let rp_dir = game_dir.join("resourcepacks");
    if !rp_dir.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in fs::read_dir(&rp_dir)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().into_owned();
        let size_bytes = if path.is_dir() {
            calculate_directory_size(&path)
        } else {
            entry.metadata()?.len()
        };

        let is_disabled = filename.ends_with(".disabled");
        packs.push(ResourcePackInfo {
            filename,
            enabled: !is_disabled,
            size_bytes,
            path,
        });
    }
    packs.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(packs)
}

fn calculate_directory_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_dir() {
                    total += calculate_directory_size(&entry.path());
                } else {
                    total += meta.len();
                }
            }
        }
    }
    total
}

fn validate_filename(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.chars().any(|c| c.is_control())
    {
        return Err(LauncherError::InvalidProfile(format!(
            "invalid file/folder name `{name}`"
        )));
    }
    Ok(())
}
