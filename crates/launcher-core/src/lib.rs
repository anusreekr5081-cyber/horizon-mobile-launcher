#![forbid(unsafe_code)]

//! The safe Rust domain layer shared by the native Android launcher and the
//! Horizon Android game host. It intentionally has no foreign VM launcher.

pub mod content;
pub mod diagnostics;
pub mod download;
pub mod error;
pub mod horizon;
pub mod install;
pub mod launcher;
pub mod model;
pub mod performance;
pub mod store;
pub mod touch;

pub use content::{list_mods, list_resource_packs, list_worlds, toggle_mod, ModInfo, WorldInfo};
pub use diagnostics::{LogCollector, SystemDiagnostics};
pub use error::{LauncherError, Result};
pub use horizon::{AndroidAbi, GAME_VERSION};
pub use install::{HorizonInstallation, InstallStatus};
pub use launcher::{LauncherSnapshot, NativeLauncher};
pub use performance::{ParticleQuality, PerformancePreset, PerformanceProfile};
pub use touch::{ButtonShape, TouchAction, TouchLayout, VirtualButton};

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Cursor,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        content::{list_mods, toggle_mod},
        diagnostics::{LogCollector, SystemDiagnostics},
        download::stage_verified_copy,
        horizon::AndroidAbi,
        model::Renderer,
        performance::PerformanceProfile,
        touch::{TouchAction, TouchLayout, VirtualButton},
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

    #[test]
    fn touch_layout_validations_and_presets() {
        let default_layout = TouchLayout::default_phone();
        assert!(!default_layout.buttons.is_empty());

        let compact = TouchLayout::compact_low_end();
        assert!(!compact.haptic_feedback); // battery saver

        let invalid_btn = VirtualButton::new("test", "T", TouchAction::Jump, 1.5, 0.5, 0.1, 0.1);
        assert!(invalid_btn.is_err());
    }

    #[test]
    fn performance_recommendation_and_limits() {
        let ultra_low = PerformanceProfile::recommend(2048, 4);
        assert_eq!(ultra_low.memory_limit_mib, 512);
        assert_eq!(ultra_low.render_distance, 4);
        assert!(ultra_low.force_16bit_depth);

        let balanced = PerformanceProfile::recommend(6144, 8);
        assert_eq!(balanced.memory_limit_mib, 1024);
        assert_eq!(balanced.target_fps, 60);

        let mut invalid_profile = ultra_low.clone();
        invalid_profile.memory_limit_mib = 100; // too low
        assert!(invalid_profile.validate().is_err());
    }

    #[test]
    fn content_mod_toggle() {
        let dir = temporary_directory("content");
        let mods_dir = dir.join("mods");
        fs::create_dir_all(&mods_dir).unwrap();

        let mod_file = mods_dir.join("sodium-horizon.jar");
        fs::write(&mod_file, b"dummy mod content").unwrap();

        let mods = list_mods(&dir).unwrap();
        assert_eq!(mods.len(), 1);
        assert!(mods[0].enabled);

        // Disable mod
        let disabled_path = toggle_mod(&dir, "sodium-horizon.jar", false).unwrap();
        assert!(disabled_path.ends_with("sodium-horizon.jar.disabled"));

        let mods_after = list_mods(&dir).unwrap();
        assert_eq!(mods_after.len(), 1);
        assert!(!mods_after[0].enabled);

        // Re-enable mod
        let reenabled = toggle_mod(&dir, "sodium-horizon.jar.disabled", true).unwrap();
        assert!(reenabled.ends_with("sodium-horizon.jar"));

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn diagnostics_and_log_sanitization() {
        let sys = SystemDiagnostics::probe();
        assert_eq!(sys.horizon_target, "26.3");
        assert!(sys.cpu_cores >= 1);

        let collector = LogCollector::new(5);
        collector.append("Normal info message");
        collector.append("Header Authorization: Bearer secret_token_12345");

        let lines = collector.snapshot();
        assert_eq!(lines.len(), 2);
        assert!(lines[1].contains("[REDACTED_TOKEN]"));
        assert!(!lines[1].contains("secret_token_12345"));
    }

    #[test]
    fn pojav_touchpad_layout_and_extreme_perf_validation() {
        let pojav = TouchLayout::pojav_touchpad();
        assert!(pojav.touchpad.is_some());
        let tp = pojav.touchpad.as_ref().unwrap();
        assert!(tp.enabled);
        assert!(tp.contains_point(0.50, 0.50));
        assert!(!tp.contains_point(0.01, 0.01));

        // Hotbar buttons 1 to 9 present
        for slot in 1..=9 {
            assert!(pojav
                .buttons
                .iter()
                .any(|b| b.action == TouchAction::HotbarSlot(slot)));
        }

        // Extreme profile testing for 120Hz/240Hz flagships
        let extreme = PerformanceProfile::extreme();
        assert_eq!(extreme.target_fps, 120);
        assert_eq!(extreme.memory_limit_mib, 4096);
        assert!(extreme.validate().is_ok());

        // Custom profile testing
        let custom = PerformanceProfile::custom(3072, 0.85, 10, 144);
        assert!(custom.is_ok());
        let custom = custom.unwrap();
        assert_eq!(custom.target_fps, 144);
        assert_eq!(custom.memory_limit_mib, 3072);
    }
}
