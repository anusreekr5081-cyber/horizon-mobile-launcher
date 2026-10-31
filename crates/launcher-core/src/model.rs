use std::{
    fmt,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{LauncherError, Result},
    store::safe_id,
};

/// Rendering backends are selected by the Android runtime bridge. The core only
/// persists the choice; it never loads a driver or executes a shell command.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Renderer {
    Auto,
    Zink,
    Virgl,
    Gl4es,
    Angle,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Zink => "zink",
            Self::Virgl => "virgl",
            Self::Gl4es => "gl4es",
            Self::Angle => "angle",
        })
    }
}

impl FromStr for Renderer {
    type Err = LauncherError;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "zink" => Ok(Self::Zink),
            "virgl" => Ok(Self::Virgl),
            "gl4es" => Ok(Self::Gl4es),
            "angle" => Ok(Self::Angle),
            _ => Err(LauncherError::InvalidProfile(format!(
                "unknown renderer `${value}`"
            ))),
        }
    }
}

/// A profile deliberately contains no account or access token. Authentication is
/// stored without account tokens or runtime settings.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub version: String,
    pub game_dir: PathBuf,
    pub memory_mib: u32,
    pub renderer: Renderer,
    pub created_unix_ms: u128,
}

impl Profile {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        game_dir: PathBuf,
        renderer: Renderer,
        memory_mib: u32,
    ) -> Result<Self> {
        let name = name.into().trim().to_owned();
        let version = version.into().trim().to_owned();
        validate_name(&name)?;
        validate_version(&version)?;
        if !(256..=16_384).contains(&memory_mib) {
            return Err(LauncherError::InvalidProfile(
                "memory must be between 256 and 16384 MiB".into(),
            ));
        }

        let created_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let id = format!("{}-{created_unix_ms}", safe_id(&name));

        Ok(Self {
            id,
            name,
            version,
            game_dir,
            memory_mib,
            renderer,
            created_unix_ms,
        })
    }
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 48 {
        return Err(LauncherError::InvalidProfile(
            "name must contain 1 to 48 characters".into(),
        ));
    }
    if safe_id(name).is_empty() {
        return Err(LauncherError::InvalidProfile(
            "name needs at least one letter or number".into(),
        ));
    }
    if name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\')) {
        return Err(LauncherError::InvalidProfile(
            "name cannot contain path separators or control characters".into(),
        ));
    }
    Ok(())
}

pub fn validate_version(version: &str) -> Result<()> {
    if version != "26.3" {
        return Err(LauncherError::InvalidProfile(
            "this app supports Horizon 26.3 only".into(),
        ));
    }
    Ok(())
}
