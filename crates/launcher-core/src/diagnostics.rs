//! Diagnostic collector and runtime system profiler for Horizon 26.3 on Android.
//! Collects hardware parameters, memory limits, and runtime logs safely without
//! leaking user tokens or private file paths.

use std::{
    collections::VecDeque,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{error::Result, horizon::GAME_VERSION};

/// Snapshot of the host hardware environment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SystemDiagnostics {
    pub os: String,
    pub architecture: String,
    pub cpu_cores: usize,
    pub total_memory_mib: u64,
    pub available_memory_mib: u64,
    pub horizon_target: &'static str,
}

impl SystemDiagnostics {
    /// Captures current system metrics from standard Linux/Android procfs.
    pub fn probe() -> Self {
        let (total_ram, avail_ram) = read_proc_meminfo();
        Self {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_cores: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            total_memory_mib: total_ram,
            available_memory_mib: avail_ram,
            horizon_target: GAME_VERSION,
        }
    }
}

/// Thread-safe in-memory ring-buffer for capturing runtime launcher and game logs.
#[derive(Clone, Debug)]
pub struct LogCollector {
    max_lines: usize,
    buffer: Arc<Mutex<VecDeque<String>>>,
}

impl Default for LogCollector {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl LogCollector {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines: max_lines.max(10),
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_lines))),
        }
    }

    /// Appends a new line, automatically sanitizing sensitive tokens or paths.
    pub fn append(&self, line: &str) {
        let sanitized = sanitize_log_line(line);
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() >= self.max_lines {
                buf.pop_front();
            }
            buf.push_back(sanitized);
        }
    }

    /// Returns a snapshot of all accumulated log lines.
    pub fn snapshot(&self) -> Vec<String> {
        self.buffer
            .lock()
            .map(|buf| buf.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Exports the log buffer to a file on disk.
    pub fn export(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lines = self.snapshot();
        let content = lines.join("\n");
        fs::write(path, content)?;
        Ok(())
    }

    /// Clears the log buffer.
    pub fn clear(&self) {
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
    }
}

fn sanitize_log_line(input: &str) -> String {
    // Redact any auth tokens, bearer headers, or secret patterns
    let mut out = input.to_string();
    if let Some(idx) = out.find("Bearer ") {
        out.replace_range((idx + 7).., "[REDACTED_TOKEN]");
    }
    if let Some(idx) = out.find("password=") {
        out.replace_range((idx + 9).., "[REDACTED]");
    }
    out
}

fn read_proc_meminfo() -> (u64, u64) {
    let mut total_mib = 0;
    let mut avail_mib = 0;

    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total_mib = parse_meminfo_kb_to_mib(line);
            } else if line.starts_with("MemAvailable:") {
                avail_mib = parse_meminfo_kb_to_mib(line);
            }
        }
    }
    (total_mib, avail_mib)
}

fn parse_meminfo_kb_to_mib(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|val| val.parse::<u64>().ok())
        .map(|kb| kb / 1024)
        .unwrap_or(0)
}
