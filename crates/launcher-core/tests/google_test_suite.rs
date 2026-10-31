//! Comprehensive test suite implemented using Google's official Rust testing framework (`googletest`).
//! Inspired by GoogleTest (C++) and Google Android CTS native test patterns.

use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use googletest::prelude::*;
use horizon_mobile_core::{
    content::{list_mods, toggle_mod},
    horizon::AndroidAbi,
    model::{validate_name, validate_version, Profile, Renderer},
    performance::{ParticleQuality, PerformancePreset, PerformanceProfile},
    touch::{TouchAction, TouchLayout, VirtualButton},
    NativeLauncher,
};

fn create_temp_dir(suffix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("gtest-horizon-{suffix}-{nonce}"));
    let _ = fs::create_dir_all(&path);
    path
}

#[googletest::test]
fn gtest_profile_name_security_and_traversal_rejection() {
    // Valid profile names
    assert_that!(validate_name("Survival 26.3"), ok(anything()));
    assert_that!(validate_name("Phone_Balanced"), ok(anything()));
    assert_that!(validate_name("LowEnd-1"), ok(anything()));

    // Path traversal attacks must be rejected
    assert_that!(validate_name("../etc/passwd"), err(anything()));
    assert_that!(validate_name(".."), err(anything()));
    assert_that!(validate_name("sub/dir"), err(anything()));
    assert_that!(validate_name("sub\\dir"), err(anything()));
    assert_that!(validate_name("profile..escape"), err(anything()));

    // Control characters and empty names
    assert_that!(validate_name(""), err(anything()));
    assert_that!(validate_name("bad\nname"), err(anything()));
    assert_that!(validate_name("   "), err(anything()));
}

#[googletest::test]
fn gtest_horizon_version_lock() {
    // Only 26.3 is supported
    assert_that!(validate_version("26.3"), ok(anything()));
    assert_that!(validate_version("1.21.4"), err(anything()));
    assert_that!(validate_version("26.4"), err(anything()));
    assert_that!(validate_version(""), err(anything()));
}

#[googletest::test]
fn gtest_profile_memory_bounds() {
    let dir = PathBuf::from("/tmp");

    // Valid memory range (256 MiB to 16384 MiB)
    let min_profile = Profile::new("MinRAM", "26.3", dir.clone(), Renderer::Auto, 256);
    assert_that!(min_profile, ok(anything()));

    let max_profile = Profile::new("MaxRAM", "26.3", dir.clone(), Renderer::Auto, 16384);
    assert_that!(max_profile, ok(anything()));

    // Out of bounds memory rejected
    let too_low = Profile::new("TooLow", "26.3", dir.clone(), Renderer::Auto, 128);
    assert_that!(too_low, err(anything()));

    let too_high = Profile::new("TooHigh", "26.3", dir, Renderer::Auto, 32768);
    assert_that!(too_high, err(anything()));
}

#[googletest::test]
fn gtest_low_end_performance_presets() {
    // Low-end budget device (2GB RAM, 4 cores)
    let budget = PerformanceProfile::recommend(2048, 4);
    expect_that!(budget.preset, eq(PerformancePreset::UltraLow));
    expect_that!(budget.memory_limit_mib, eq(512));
    expect_that!(budget.render_scale, eq(0.50));
    expect_that!(budget.render_distance, eq(4));
    expect_that!(budget.target_fps, eq(30));
    expect_that!(budget.max_worker_threads, eq(2));
    expect_that!(budget.force_16bit_depth, eq(true));
    expect_that!(budget.particles, eq(ParticleQuality::Minimal));

    // Mid-range device (6GB RAM, 8 cores)
    let mid = PerformanceProfile::recommend(6144, 8);
    expect_that!(mid.preset, eq(PerformancePreset::Balanced));
    expect_that!(mid.memory_limit_mib, eq(1024));
    expect_that!(mid.render_scale, eq(0.75));
    expect_that!(mid.target_fps, eq(60));

    // Fallback on zero / unknown detection
    let unknown = PerformanceProfile::recommend(0, 0);
    expect_that!(unknown.preset, eq(PerformancePreset::UltraLow));
}

#[googletest::test]
fn gtest_touch_controls_validation_and_hit_testing() {
    let button = VirtualButton::new(
        "btn_pri",
        "PRI",
        TouchAction::AttackMine,
        0.70,
        0.60,
        0.15,
        0.15,
    );
    assert_that!(button, ok(anything()));
    let btn = button.unwrap();

    // Hit testing inside button boundaries
    assert_that!(btn.contains_point(0.75, 0.65), eq(true));
    assert_that!(btn.contains_point(0.70, 0.60), eq(true));

    // Hit testing outside
    assert_that!(btn.contains_point(0.10, 0.10), eq(false));
    assert_that!(btn.contains_point(0.90, 0.90), eq(false));

    // Preset configurations
    let phone_layout = TouchLayout::default_phone();
    expect_that!(phone_layout.buttons.len(), ge(10));
    expect_that!(phone_layout.scale, eq(1.0));

    let compact_layout = TouchLayout::compact_low_end();
    expect_that!(compact_layout.haptic_feedback, eq(false)); // Disabled for low-end battery saving
}

#[googletest::test]
fn gtest_isolated_store_lifecycle() {
    let test_dir = create_temp_dir("store");
    let launcher = NativeLauncher::open(test_dir.clone(), AndroidAbi::Arm64V8a);
    assert_that!(launcher, ok(anything()));
    let launcher = launcher.unwrap();

    // Create profile
    let profile = launcher
        .profiles()
        .create_profile("Benchmark", "26.3", Renderer::Gl4es, 768);
    assert_that!(profile, ok(anything()));
    let profile = profile.unwrap();
    expect_that!(profile.name, eq("Benchmark"));
    expect_that!(profile.renderer, eq(Renderer::Gl4es));

    // Verify instance directories created
    expect_that!(profile.game_dir.join("saves").is_dir(), eq(true));
    expect_that!(profile.game_dir.join("mods").is_dir(), eq(true));
    expect_that!(profile.game_dir.join("resourcepacks").is_dir(), eq(true));

    // Duplicate profile name rejection
    let dup = launcher
        .profiles()
        .create_profile("Benchmark", "26.3", Renderer::Auto, 512);
    assert_that!(dup, err(anything()));

    // List and delete
    let list = launcher.profiles().list_profiles();
    assert_that!(list, ok(anything()));
    expect_that!(list.unwrap().len(), eq(1));

    let deleted = launcher.profiles().delete_profile(&profile.id);
    assert_that!(deleted, ok(anything()));

    let _ = fs::remove_dir_all(test_dir);
}

#[googletest::test]
fn gtest_content_mods_management() {
    let test_dir = create_temp_dir("mods");
    let mods_dir = test_dir.join("mods");
    let _ = fs::create_dir_all(&mods_dir);

    let mod_path = mods_dir.join("perf-opt.jar");
    let _ = fs::write(&mod_path, b"test jar");

    let mods = list_mods(&test_dir);
    assert_that!(mods, ok(anything()));
    let mods = mods.unwrap();
    expect_that!(mods.len(), eq(1));
    expect_that!(mods[0].enabled, eq(true));

    // Toggle off
    let disabled = toggle_mod(&test_dir, "perf-opt.jar", false);
    assert_that!(disabled, ok(anything()));

    let mods_disabled = list_mods(&test_dir).unwrap();
    expect_that!(mods_disabled[0].enabled, eq(false));
    expect_that!(mods_disabled[0].filename, ends_with(".disabled"));

    let _ = fs::remove_dir_all(test_dir);
}

#[googletest::test]
fn gtest_pojav_touchpad_and_hotbar_validation() {
    let pojav = TouchLayout::pojav_touchpad();
    assert_that!(pojav.touchpad.is_some(), eq(true));
    let tp = pojav.touchpad.as_ref().unwrap();
    expect_that!(tp.enabled, eq(true));
    expect_that!(tp.contains_point(0.50, 0.50), eq(true));
    expect_that!(tp.contains_point(0.01, 0.01), eq(false));

    // Must have Pojav virtual mouse button controls
    let has_mouse = pojav
        .buttons
        .iter()
        .any(|b| b.action == TouchAction::ToggleMouseMode);
    let has_pri = pojav
        .buttons
        .iter()
        .any(|b| b.action == TouchAction::AttackMine);
    let has_sec = pojav
        .buttons
        .iter()
        .any(|b| b.action == TouchAction::UsePlace);
    let has_mid = pojav
        .buttons
        .iter()
        .any(|b| b.action == TouchAction::PickBlockMid);

    expect_that!(has_mouse, eq(true));
    expect_that!(has_pri, eq(true));
    expect_that!(has_sec, eq(true));
    expect_that!(has_mid, eq(true));

    // Hotbar slots 1 through 9
    for slot in 1..=9 {
        let has_slot = pojav
            .buttons
            .iter()
            .any(|b| b.action == TouchAction::HotbarSlot(slot));
        expect_that!(has_slot, eq(true));
    }
}

#[googletest::test]
fn gtest_extreme_and_custom_performance_tuning() {
    // Extreme preset for high-refresh 120Hz displays
    let extreme = PerformanceProfile::extreme();
    expect_that!(extreme.preset, eq(PerformancePreset::Extreme));
    expect_that!(extreme.target_fps, eq(120));
    expect_that!(extreme.memory_limit_mib, eq(4096));
    assert_that!(extreme.validate(), ok(anything()));

    // Custom configuration with custom FPS and scale
    let custom = PerformanceProfile::custom(3072, 0.80, 10, 144);
    assert_that!(custom, ok(anything()));
    let custom = custom.unwrap();
    expect_that!(custom.preset, eq(PerformancePreset::Custom));
    expect_that!(custom.target_fps, eq(144));
    expect_that!(custom.render_scale, eq(0.80));

    // Invalid custom bounds
    let invalid_fps = PerformanceProfile::custom(1024, 0.75, 8, 300);
    assert_that!(invalid_fps, err(anything()));

    let invalid_ram = PerformanceProfile::custom(64, 0.75, 8, 60);
    assert_that!(invalid_ram, err(anything()));
}
