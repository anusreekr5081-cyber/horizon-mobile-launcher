//! Dynamic Native Bridge for loading and executing Horizon 26.3 client
//! libraries directly inside Android's native process space without a JVM.

use std::{
    ffi::{c_char, c_void, CString},
    path::{Path, PathBuf},
};

use horizon_mobile_core::{model::Renderer, performance::PerformanceProfile, touch::TouchLayout};

/// Parameters passed to the native Horizon 26.3 game loop.
#[derive(Clone, Debug)]
pub struct GameLaunchConfig {
    pub game_dir: PathBuf,
    pub renderer: Renderer,
    pub performance: PerformanceProfile,
    pub touch_layout: TouchLayout,
}

/// Function signature expected by the Horizon 26.3 Android native client library.
pub type HorizonMainFn = unsafe extern "C" fn(
    game_dir_utf8: *const c_char,
    renderer_utf8: *const c_char,
    render_scale: f32,
    memory_limit_mib: u32,
    native_window: *mut c_void,
) -> i32;

/// Manages dynamic loading and lifecycle dispatch of `libhorizon_client.so`.
#[derive(Debug)]
pub struct NativeClientBridge {
    library_path: PathBuf,
}

impl NativeClientBridge {
    pub fn new(library_path: impl Into<PathBuf>) -> Self {
        Self {
            library_path: library_path.into(),
        }
    }

    pub fn library_path(&self) -> &Path {
        &self.library_path
    }

    /// Checks if the compiled Horizon 26.3 native library binary is present.
    pub fn is_installed(&self) -> bool {
        self.library_path.is_file()
    }

    /// Invokes the native game entry point with the given configuration and Android window handle.
    ///
    /// # Safety
    /// This function calls into foreign native code exported by `libhorizon_client.so`.
    pub unsafe fn launch(
        &self,
        config: &GameLaunchConfig,
        native_window: *mut c_void,
    ) -> Result<i32, String> {
        if !self.is_installed() {
            return Err(format!(
                "Horizon 26.3 native library not found at {}",
                self.library_path.display()
            ));
        }

        let game_dir_str = config.game_dir.to_str().ok_or("invalid game_dir path")?;
        let game_dir_c = CString::new(game_dir_str).map_err(|e| e.to_string())?;

        let renderer_str = config.renderer.to_string();
        let renderer_c = CString::new(renderer_str).map_err(|e| e.to_string())?;

        #[cfg(target_os = "android")]
        {
            let lib_c = CString::new(self.library_path.to_str().unwrap()).unwrap();
            let handle = libc::dlopen(lib_c.as_ptr(), libc::RTLD_NOW);
            if handle.is_null() {
                let err_ptr = libc::dlerror();
                let err_str = if !err_ptr.is_null() {
                    std::ffi::CStr::from_ptr(err_ptr)
                        .to_string_lossy()
                        .into_owned()
                } else {
                    "unknown dlopen error".to_string()
                };
                return Err(format!("dlopen failed: {err_str}"));
            }

            let symbol_name = CString::new("horizon_client_android_main").unwrap();
            let func_ptr = libc::dlsym(handle, symbol_name.as_ptr());
            if func_ptr.is_null() {
                libc::dlclose(handle);
                return Err("symbol `horizon_client_android_main` not found in library".into());
            }

            let entry: HorizonMainFn = std::mem::transmute(func_ptr);
            let code = entry(
                game_dir_c.as_ptr(),
                renderer_c.as_ptr(),
                config.performance.render_scale,
                config.performance.memory_limit_mib,
                native_window,
            );

            libc::dlclose(handle);
            Ok(code)
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = (game_dir_c, renderer_c, native_window);
            // On non-Android hosts (tests/CI), confirm validation passed cleanly.
            Ok(0)
        }
    }
}
