use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{LauncherError, Result},
    model::{Profile, Renderer},
};

const PROFILES_FILE: &str = "profiles.json";

#[derive(Debug)]
pub struct LauncherStore {
    root: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
struct PersistedProfiles {
    schema: u32,
    profiles: Vec<Profile>,
}

impl LauncherStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        fs::create_dir_all(root.join("instances"))?;
        fs::create_dir_all(root.join("versions"))?;
        fs::create_dir_all(root.join("runtimes"))?;
        fs::create_dir_all(root.join("downloads"))?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn list_profiles(&self) -> Result<Vec<Profile>> {
        let path = self.profiles_path();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let saved: PersistedProfiles = serde_json::from_slice(&fs::read(path)?)?;
        if saved.schema != 1 {
            return Err(LauncherError::InvalidProfile(format!(
                "unsupported profile database schema {}",
                saved.schema
            )));
        }
        for profile in &saved.profiles {
            self.validate_loaded_profile(profile)?;
        }
        Ok(saved.profiles)
    }

    pub fn create_profile(
        &self,
        name: impl Into<String>,
        version: impl Into<String>,
        renderer: Renderer,
        memory_mib: u32,
    ) -> Result<Profile> {
        let name = name.into();
        let version = version.into();
        let provisional = Profile::new(&name, &version, PathBuf::new(), renderer, memory_mib)?;
        let id = self.next_available_id(provisional.id);
        let profile = Profile {
            game_dir: self.instance_dir(&id),
            id,
            ..provisional
        };

        let mut profiles = self.list_profiles()?;
        if profiles
            .iter()
            .any(|existing| existing.name.eq_ignore_ascii_case(&profile.name))
        {
            return Err(LauncherError::InvalidProfile(
                "a profile with that name already exists".into(),
            ));
        }

        fs::create_dir_all(profile.game_dir.join("saves"))?;
        fs::create_dir_all(profile.game_dir.join("resourcepacks"))?;
        fs::create_dir_all(profile.game_dir.join("mods"))?;
        profiles.push(profile.clone());
        self.save_profiles(&profiles)?;
        Ok(profile)
    }

    pub fn delete_profile(&self, id: &str) -> Result<Option<Profile>> {
        let mut profiles = self.list_profiles()?;
        let Some(index) = profiles.iter().position(|profile| profile.id == id) else {
            return Ok(None);
        };
        let profile = profiles.remove(index);
        self.validate_loaded_profile(&profile)?;
        // Removing the profile never removes user worlds. The UI may move its
        // instance folder to Android's trash only after explicit confirmation.
        self.save_profiles(&profiles)?;
        Ok(Some(profile))
    }

    fn profiles_path(&self) -> PathBuf {
        self.root.join(PROFILES_FILE)
    }

    fn instance_dir(&self, id: &str) -> PathBuf {
        self.root.join("instances").join(id)
    }

    fn next_available_id(&self, base: String) -> String {
        if !self.instance_dir(&base).exists() {
            return base;
        }
        for suffix in 2.. {
            let candidate = format!("{base}-{suffix}");
            if !self.instance_dir(&candidate).exists() {
                return candidate;
            }
        }
        unreachable!("u64 suffix space cannot be exhausted")
    }

    fn validate_loaded_profile(&self, profile: &Profile) -> Result<()> {
        let prefix = safe_id(&profile.name);
        if prefix.is_empty() || !profile.id.starts_with(&format!("{prefix}-")) {
            return Err(LauncherError::InvalidProfile(
                "profile id is malformed".into(),
            ));
        }
        if profile.game_dir != self.instance_dir(&profile.id) {
            return Err(LauncherError::InvalidProfile(
                "profile game directory is outside the launcher instance directory".into(),
            ));
        }
        Ok(())
    }

    fn save_profiles(&self, profiles: &[Profile]) -> Result<()> {
        let contents = serde_json::to_vec_pretty(&PersistedProfiles {
            schema: 1,
            profiles: profiles.to_vec(),
        })?;
        let final_path = self.profiles_path();
        let temporary_path = self
            .root
            .join(format!(".{PROFILES_FILE}.{}.tmp", std::process::id()));
        {
            let mut file = fs::File::create(&temporary_path)?;
            file.write_all(&contents)?;
            file.sync_all()?;
        }
        fs::rename(temporary_path, final_path)?;
        Ok(())
    }
}

/// Produces an ASCII-only path component. UI code can preview IDs, but only the
/// launcher store is allowed to map an ID to a filesystem path.
pub fn safe_id(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut previous_dash = false;

    for character in value.chars() {
        let next = if character.is_ascii_alphanumeric() {
            previous_dash = false;
            character.to_ascii_lowercase()
        } else if !previous_dash {
            previous_dash = true;
            '-'
        } else {
            continue;
        };
        result.push(next);
    }

    result.trim_matches('-').to_owned()
}
