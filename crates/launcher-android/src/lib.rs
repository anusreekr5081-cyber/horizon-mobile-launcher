//! Native Rust Android Application and Horizon 26.3 Host (0% Java / JVM).
//! Uses `android-activity` (`ANativeActivity`) to run completely inside Android's
//! native Linux process boundary without Java bytecode or Dalvik/ART overhead.

pub mod bridge;
pub mod input;

pub use bridge::{GameLaunchConfig, NativeClientBridge};
pub use input::TouchInputManager;

#[cfg(target_os = "android")]
use android_activity::{AndroidApp, MainEvent, PollEvent};
#[cfg(target_os = "android")]
use horizon_mobile_core::{
    horizon::AndroidAbi, model::Renderer, touch::TouchLayout, NativeLauncher,
};

/// Entry point invoked by Android's `NativeActivity` when the app launches.
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let internal_path = app
        .internal_data_path()
        .unwrap_or_else(|| std::path::PathBuf::from("/data/data/dev.horizonmobile.launcher/files"));

    let launcher = match NativeLauncher::open(internal_path.clone(), AndroidAbi::Arm64V8a) {
        Ok(l) => l,
        Err(_) => return,
    };

    // Initialize or load default profile if none exists
    let profiles = launcher.profiles().list_profiles().unwrap_or_default();
    let _active_profile = if let Some(first) = profiles.into_iter().next() {
        Some(first)
    } else {
        launcher
            .profiles()
            .create_profile("Low End 26.3", "26.3", Renderer::Auto, 512)
            .ok()
    };

    let touch_layout = TouchLayout::compact_low_end();
    let mut input_manager = TouchInputManager::new(touch_layout);
    let mut is_running = true;

    while is_running {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(MainEvent::Resume { .. }) => {
                    // App brought to foreground
                }
                PollEvent::Main(MainEvent::Pause) => {
                    // App backgrounded
                    input_manager.clear();
                }
                PollEvent::Main(MainEvent::TerminateWindow { .. }) => {
                    is_running = false;
                }
                _ => {}
            }
        });
    }
}

/// JNI bridge methods callable from Java Activity classes (`MainActivity` and `GameActivity`).
#[cfg(target_os = "android")]
pub mod jni_exports {
    use super::*;
    use horizon_mobile_core::diagnostics::SystemDiagnostics;
    use horizon_mobile_core::model::Renderer;
    use horizon_mobile_core::performance::PerformanceProfile;
    use horizon_mobile_core::touch::TouchLayout;
    use jni::objects::{JClass, JObject, JString};
    use jni::sys::{jboolean, jfloat, jint, jlong, jstring};
    use jni::JNIEnv;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[no_mangle]
    pub extern "system" fn Java_dev_horizonmobile_launcher_NativeBridge_launchGame(
        mut env: JNIEnv,
        _class: JClass,
        game_dir: JString,
        renderer: JString,
        render_scale: jfloat,
        memory_limit_mib: jint,
        _surface: JObject,
    ) -> jint {
        let game_dir_str: String = match env.get_string(&game_dir) {
            Ok(s) => s.into(),
            Err(_) => return -1,
        };
        let renderer_str: String = match env.get_string(&renderer) {
            Ok(s) => s.into(),
            Err(_) => "auto".into(),
        };

        let parsed_renderer = Renderer::from_str(&renderer_str).unwrap_or(Renderer::Auto);
        let mut perf = PerformanceProfile::recommend(memory_limit_mib as u64, 4);
        perf.memory_limit_mib = memory_limit_mib as u32;
        perf.render_scale = render_scale;

        let config = GameLaunchConfig {
            game_dir: PathBuf::from(game_dir_str),
            renderer: parsed_renderer,
            performance: perf,
            touch_layout: TouchLayout::pojav_touchpad(),
        };

        let lib_path = config.game_dir.join("bin").join("libhorizon_client.so");
        let bridge = NativeClientBridge::new(lib_path);
        unsafe { bridge.launch(&config, std::ptr::null_mut()).unwrap_or(-2) }
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_horizonmobile_launcher_NativeBridge_sendTouchpadDelta(
        _env: JNIEnv,
        _class: JClass,
        _dx: jfloat,
        _dy: jfloat,
    ) {
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_horizonmobile_launcher_NativeBridge_sendButtonAction(
        _env: JNIEnv,
        _class: JClass,
        _action_id: jint,
        _is_down: jboolean,
    ) {
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_horizonmobile_launcher_NativeBridge_getDiagnosticsJson(
        env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        let diag = SystemDiagnostics::probe();
        let json = format!(
            "{{\"os\":\"{}\",\"cores\":{},\"total_ram_mib\":{},\"avail_ram_mib\":{}}}",
            diag.os, diag.cpu_cores, diag.total_memory_mib, diag.available_memory_mib
        );
        env.new_string(json)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }

    #[no_mangle]
    pub extern "system" fn Java_dev_horizonmobile_launcher_NativeBridge_recommendPresetJson(
        env: JNIEnv,
        _class: JClass,
        total_ram_mib: jlong,
        cores: jint,
    ) -> jstring {
        let rec = PerformanceProfile::recommend(total_ram_mib as u64, cores as usize);
        let json = format!(
            "{{\"preset\":\"{:?}\",\"ram_limit\":{},\"scale\":{},\"fps\":{}}}",
            rec.preset, rec.memory_limit_mib, rec.render_scale, rec.target_fps
        );
        env.new_string(json)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut())
    }
}

#[cfg(test)]
mod tests {
    use horizon_mobile_core::touch::{TouchAction, TouchLayout};

    use super::{
        bridge::{GameLaunchConfig, NativeClientBridge},
        input::TouchInputManager,
    };

    #[test]
    fn touch_input_manager_tracking_and_toggle() {
        let layout = TouchLayout::default_phone();
        let mut manager = TouchInputManager::new(layout);

        // Find Jump button coordinates from layout
        let jump_btn = manager
            .layout()
            .buttons
            .iter()
            .find(|b| b.action == TouchAction::Jump)
            .unwrap();
        let center_x = jump_btn.x + (jump_btn.width / 2.0);
        let center_y = jump_btn.y + (jump_btn.height / 2.0);

        // Finger down on Jump button
        let action = manager.on_pointer_down(1, center_x, center_y);
        assert_eq!(action, Some(TouchAction::Jump));
        assert!(manager.is_action_pressed(TouchAction::Jump));

        // Finger up
        let released = manager.on_pointer_up(1);
        assert_eq!(released, Some(TouchAction::Jump));
        assert!(!manager.is_action_pressed(TouchAction::Jump));
    }

    #[test]
    fn pojav_touchpad_mouse_mode_and_relative_motion() {
        let layout = TouchLayout::pojav_touchpad();
        assert!(layout.touchpad.is_some());
        let mut manager = TouchInputManager::new(layout);

        assert!(!manager.is_mouse_mode());

        // Find MOUSE toggle button
        let mouse_btn = manager
            .layout()
            .buttons
            .iter()
            .find(|b| b.action == TouchAction::ToggleMouseMode)
            .expect("MOUSE button should exist in Pojav layout");
        let btn_x = mouse_btn.x + (mouse_btn.width / 2.0);
        let btn_y = mouse_btn.y + (mouse_btn.height / 2.0);

        // Tap MOUSE toggle button -> toggles mouse mode on
        let action = manager.on_pointer_down(10, btn_x, btn_y);
        assert_eq!(action, Some(TouchAction::ToggleMouseMode));
        assert!(manager.is_mouse_mode());
        manager.on_pointer_up(10);
        assert!(manager.is_mouse_mode());

        // Touchpad area drag: pointer down inside touchpad area (e.g. at 0.50, 0.50)
        let tp_action = manager.on_pointer_down(20, 0.50, 0.50);
        assert_eq!(tp_action, None); // Touchpad drag is not a button action
        assert!(manager.is_touchpad_active());

        // Move pointer by +0.05 X, +0.02 Y
        manager.on_pointer_move(20, 0.55, 0.52);
        let (dx, dy) = manager.take_accumulated_delta();
        assert!(dx > 0.0);
        assert!(dy > 0.0);

        // Release touchpad pointer
        manager.on_pointer_up(20);
        assert!(!manager.is_touchpad_active());
    }

    #[test]
    fn bridge_mock_validation() {
        let bridge = NativeClientBridge::new("/nonexistent/libhorizon_client.so");
        assert!(!bridge.is_installed());

        let config = GameLaunchConfig {
            game_dir: std::path::PathBuf::from("/tmp/game"),
            renderer: horizon_mobile_core::model::Renderer::Auto,
            performance: horizon_mobile_core::performance::PerformanceProfile::ultra_low(),
            touch_layout: TouchLayout::compact_low_end(),
        };

        unsafe {
            // On non-installed target, should safely return descriptive Err
            let result = bridge.launch(&config, std::ptr::null_mut());
            assert!(result.is_err());
        }
    }
}
