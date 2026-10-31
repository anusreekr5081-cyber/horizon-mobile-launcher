#![forbid(unsafe_code)]

//! The safe Rust domain layer shared by the native Android launcher and the
//! Horizon Android game host. It intentionally has no foreign VM launcher.

pub mod download;
pub mod error;
pub mod horizon;
pub mod install;
pub mod launcher;
pub mod model;
pub mod store;

pub use error::{LauncherError, Result};
pub use horizon::{AndroidAbi, GAME_VERSION};
pub use install::{HorizonInstallation, InstallStatus};
pub use launcher::{LauncherSnapshot, NativeLauncher};

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor, time::{SystemTime, UNIX_EPOCH}};

    use super::{
        download::stage_verified_copy,
        horizon::AndroidAbi,
        model::Renderer,
        NativeLauncher,
    };

    fn temporary_directory(name: &str) -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("horizon-mobile-{name}-{timestamp}"))
    }

    #[test]
    fn profiles_are_fixed_to_horizon_26_3() {
        let directory = temporary_directory("profiles");
        let launcher = NativeLauncher::open(directory.clone(), AndroidAbi::Arm64V8a).unwrap();

        let profile = launcher
            .profiles()
            .create_profile("Low end", "26.3", Renderer::Auto, 512)
            .unwrap();
        assert_eq!(profile.version, "26.3");
        assert!(launcher
            .profiles()
            .create_profile("Other", "1.21.4", Renderer::Auto, 512)
            .is_err());

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn checksum_failure_never_replaces_the_target() {
        let directory = temporary_directory("download");
        let target = directory.join("payload.bin");
        let mut source = Cursor::new(b"wrong".to_vec());

        let result = stage_verified_copy(
            &mut source,
            &target,
            "a9993e364706816aba3e25717850c26c9cd0d89d",
        );
        assert!(result.is_err());
        assert!(!target.exists());

        fs::remove_dir_all(directory).unwrap();
    }
}
