//! Low-end phone performance optimizer for Horizon 26.3 on Android.
//! Calibrates memory budgets, render resolution scaling, chunk distance,
//! worker thread caps, and GPU driver flags to keep low-end phones from
//! overheating, dropping frames, or running out of memory.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{LauncherError, Result};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PerformancePreset {
    UltraLow,
    Low,
    Balanced,
    High,
    Extreme,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticleQuality {
    Minimal,
    Decreased,
    All,
}

/// Hardware-tuned performance profile for mobile execution.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PerformanceProfile {
    pub preset: PerformancePreset,
    /// Maximum memory allocated in MiB (e.g. 512, 768, 1024).
    pub memory_limit_mib: u32,
    /// Viewport render scale multiplier (e.g. 0.5 for 50% internal resolution).
    pub render_scale: f32,
    /// Chunk render distance (e.g. 4 for ultra-low, 8 for balanced).
    pub render_distance: u32,
    /// Target frame rate limit (e.g. 30, 45, 60).
    pub target_fps: u32,
    /// Thread count cap for terrain generation and meshing.
    pub max_worker_threads: usize,
    /// Vertical synchronization.
    pub vsync: bool,
    /// Particle effect density.
    pub particles: ParticleQuality,
    /// Texture mipmapping level (0 to disable).
    pub mipmaps: u32,
    /// Force 16-bit depth buffer to reduce VRAM bandwidth on budget GPUs.
    pub force_16bit_depth: bool,
    /// Enable aggressive memory reclamation during chunk unload.
    pub aggressive_gc: bool,
    /// Cache compiled shader pipelines to prevent stuttering.
    pub shader_cache: bool,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::balanced()
    }
}

impl PerformanceProfile {
    /// Ultra-Low preset specifically tuned for budget 2GB–3GB RAM devices
    /// (e.g. Snapdragon 4xx / MediaTek Helio G series).
    pub fn ultra_low() -> Self {
        Self {
            preset: PerformancePreset::UltraLow,
            memory_limit_mib: 512,
            render_scale: 0.50,
            render_distance: 4,
            target_fps: 30,
            max_worker_threads: 2,
            vsync: true,
            particles: ParticleQuality::Minimal,
            mipmaps: 0,
            force_16bit_depth: true,
            aggressive_gc: true,
            shader_cache: true,
        }
    }

    /// Low preset tuned for 3GB–4GB RAM phones.
    pub fn low() -> Self {
        Self {
            preset: PerformancePreset::Low,
            memory_limit_mib: 768,
            render_scale: 0.65,
            render_distance: 6,
            target_fps: 40,
            max_worker_threads: 2,
            vsync: true,
            particles: ParticleQuality::Decreased,
            mipmaps: 1,
            force_16bit_depth: true,
            aggressive_gc: true,
            shader_cache: true,
        }
    }

    /// Balanced preset tuned for 4GB–6GB RAM devices.
    pub fn balanced() -> Self {
        Self {
            preset: PerformancePreset::Balanced,
            memory_limit_mib: 1024,
            render_scale: 0.75,
            render_distance: 8,
            target_fps: 60,
            max_worker_threads: 4,
            vsync: true,
            particles: ParticleQuality::Decreased,
            mipmaps: 2,
            force_16bit_depth: false,
            aggressive_gc: false,
            shader_cache: true,
        }
    }

    /// High preset for upper mid-range / flagship Android devices.
    pub fn high() -> Self {
        Self {
            preset: PerformancePreset::High,
            memory_limit_mib: 2048,
            render_scale: 1.0,
            render_distance: 12,
            target_fps: 60,
            max_worker_threads: 4,
            vsync: true,
            particles: ParticleQuality::All,
            mipmaps: 4,
            force_16bit_depth: false,
            aggressive_gc: false,
            shader_cache: true,
        }
    }

    /// Extreme preset for high-refresh-rate flagship Android devices (90Hz / 120Hz / 144Hz / 240Hz).
    pub fn extreme() -> Self {
        Self {
            preset: PerformancePreset::Extreme,
            memory_limit_mib: 4096,
            render_scale: 1.0,
            render_distance: 16,
            target_fps: 120,
            max_worker_threads: 8,
            vsync: false,
            particles: ParticleQuality::All,
            mipmaps: 4,
            force_16bit_depth: false,
            aggressive_gc: false,
            shader_cache: true,
        }
    }

    /// User-customized performance configuration with strict hardware safety bounds validation.
    pub fn custom(
        memory_limit_mib: u32,
        render_scale: f32,
        render_distance: u32,
        target_fps: u32,
    ) -> Result<Self> {
        let mut profile = Self::balanced();
        profile.preset = PerformancePreset::Custom;
        profile.memory_limit_mib = memory_limit_mib;
        profile.render_scale = render_scale;
        profile.render_distance = render_distance;
        profile.target_fps = target_fps;
        profile.validate()?;
        Ok(profile)
    }

    /// Automatically suggests the safest performance preset given available device RAM and CPU cores.
    pub fn recommend(ram_mib: u64, cpu_cores: usize) -> Self {
        if ram_mib == 0 || ram_mib <= 2560 || cpu_cores <= 4 {
            Self::ultra_low()
        } else if ram_mib <= 4096 || cpu_cores <= 6 {
            Self::low()
        } else if ram_mib <= 6144 {
            Self::balanced()
        } else {
            Self::high()
        }
    }

    pub fn validate(&self) -> Result<()> {
        if !(256..=16384).contains(&self.memory_limit_mib) {
            return Err(LauncherError::InvalidProfile(
                "memory limit must be between 256 and 16384 MiB".into(),
            ));
        }
        if !(0.25..=2.0).contains(&self.render_scale) {
            return Err(LauncherError::InvalidProfile(
                "render scale must be between 0.25 and 2.0".into(),
            ));
        }
        if !(2..=32).contains(&self.render_distance) {
            return Err(LauncherError::InvalidProfile(
                "render distance must be between 2 and 32 chunks".into(),
            ));
        }
        if !(15..=240).contains(&self.target_fps) {
            return Err(LauncherError::InvalidProfile(
                "target FPS must be between 15 and 240".into(),
            ));
        }
        if self.max_worker_threads == 0 || self.max_worker_threads > 32 {
            return Err(LauncherError::InvalidProfile(
                "max worker threads must be between 1 and 32".into(),
            ));
        }
        Ok(())
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(LauncherError::MissingFile(path.to_owned()));
        }
        let data = fs::read(path)?;
        let profile: Self = serde_json::from_slice(&data)?;
        profile.validate()?;
        Ok(profile)
    }
}
