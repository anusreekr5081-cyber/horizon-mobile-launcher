//! Standalone Command Line Tool for Horizon 26.3 Mobile Launcher.
//! Allows offline profile management, installation verification, performance tuning,
//! and content inspection directly from the terminal without a GUI.

use std::{env, path::PathBuf, process::ExitCode};

use horizon_mobile_core::{
    content::{list_mods, list_worlds, toggle_mod},
    diagnostics::SystemDiagnostics,
    horizon::AndroidAbi,
    model::Renderer,
    performance::PerformanceProfile,
    touch::TouchLayout,
    NativeLauncher,
};

fn print_help() {
    println!(
        r#"Horizon 26.3 Mobile Launcher CLI

USAGE:
    horizon-cli <COMMAND> [OPTIONS]

COMMANDS:
    status                        Display launcher installation status and profiles
    diagnostics                   Display hardware diagnostics and recommended settings
    profile list                  List all configured game profiles
    profile create <NAME> [RAM]   Create a new profile (e.g. `horizon-cli profile create "LowEnd" 512`)
    profile delete <ID>           Delete a profile by ID
    perf recommend                Suggest optimal low-end settings for this device
    perf extreme                  Print 120Hz/240Hz flagship performance configuration
    perf custom <RAM> <SCL> <CHK> <FPS> Validate custom performance parameters
    touch export <PRESET> <OUT>   Export touch layout (PRESET: default | compact | pojav)
    content worlds <PROFILE_ID>   List worlds in a profile's game directory
    content mods <PROFILE_ID>     List installed mods in a profile
    content toggle <PROFILE_ID> <MOD> <on|off> Toggle a mod
"#
    );
}

fn get_data_dir() -> PathBuf {
    if let Ok(custom) = env::var("HORIZON_DATA_DIR") {
        PathBuf::from(custom)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".horizon-mobile")
    } else {
        PathBuf::from("./.horizon-mobile")
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        print_help();
        return ExitCode::SUCCESS;
    }

    let data_dir = get_data_dir();
    let launcher = match NativeLauncher::open(data_dir.clone(), AndroidAbi::Arm64V8a) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error opening launcher database at {:?}: {}", data_dir, e);
            return ExitCode::FAILURE;
        }
    };

    match args[1].as_str() {
        "status" => {
            let snap = match launcher.snapshot() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error capturing snapshot: {e}");
                    return ExitCode::FAILURE;
                }
            };
            println!("=== Horizon 26.3 Mobile Launcher Status ===");
            println!("Data Root:       {}", data_dir.display());
            println!("Install Status:  {:?}", snap.install_status);
            println!("Total Profiles:  {}", snap.profiles.len());
            for p in &snap.profiles {
                println!(
                    "  - [{}] {} (RAM: {} MiB, Renderer: {})",
                    p.id, p.name, p.memory_mib, p.renderer
                );
            }
        }
        "diagnostics" => {
            let diag = SystemDiagnostics::probe();
            println!("=== System Diagnostics ===");
            println!("OS:              {}", diag.os);
            println!("Architecture:    {}", diag.architecture);
            println!("CPU Cores:       {}", diag.cpu_cores);
            println!("Total Memory:    {} MiB", diag.total_memory_mib);
            println!("Avail Memory:    {} MiB", diag.available_memory_mib);
            println!("Target Version:  Horizon {}", diag.horizon_target);

            let rec = PerformanceProfile::recommend(diag.total_memory_mib, diag.cpu_cores);
            println!("\nRecommended Performance Preset: {:?}", rec.preset);
            println!("  - Max RAM:      {} MiB", rec.memory_limit_mib);
            println!("  - Render Scale: {:.0}%", rec.render_scale * 100.0);
            println!("  - Chunks:       {}", rec.render_distance);
            println!("  - Target FPS:   {}", rec.target_fps);
            println!("  - Worker Threads: {}", rec.max_worker_threads);
        }
        "profile" => {
            if args.len() < 3 {
                eprintln!("Missing subcommand: list, create, delete");
                return ExitCode::FAILURE;
            }
            match args[2].as_str() {
                "list" => {
                    let profiles = launcher.profiles().list_profiles().unwrap_or_default();
                    if profiles.is_empty() {
                        println!("No profiles found. Create one with `horizon-cli profile create <NAME>`.");
                    } else {
                        println!("Profiles ({}):", profiles.len());
                        for p in profiles {
                            println!("  * ID: {}\n    Name: {}\n    Memory: {} MiB\n    Renderer: {}\n    Dir: {}",
                                p.id, p.name, p.memory_mib, p.renderer, p.game_dir.display());
                        }
                    }
                }
                "create" => {
                    if args.len() < 4 {
                        eprintln!("Usage: horizon-cli profile create <NAME> [RAM_MIB] [RENDERER]");
                        return ExitCode::FAILURE;
                    }
                    let name = &args[3];
                    let ram = args.get(4).and_then(|r| r.parse().ok()).unwrap_or(512);
                    let renderer = args
                        .get(5)
                        .and_then(|r| r.parse().ok())
                        .unwrap_or(Renderer::Auto);

                    match launcher
                        .profiles()
                        .create_profile(name, "26.3", renderer, ram)
                    {
                        Ok(p) => println!("Created profile `{}` with ID: {}", p.name, p.id),
                        Err(e) => {
                            eprintln!("Failed to create profile: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                "delete" => {
                    if args.len() < 4 {
                        eprintln!("Usage: horizon-cli profile delete <ID>");
                        return ExitCode::FAILURE;
                    }
                    match launcher.profiles().delete_profile(&args[3]) {
                        Ok(Some(p)) => println!("Deleted profile: {} ({})", p.name, p.id),
                        Ok(None) => println!("Profile not found: {}", args[3]),
                        Err(e) => {
                            eprintln!("Failed to delete profile: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                other => {
                    eprintln!("Unknown profile subcommand: {other}");
                    return ExitCode::FAILURE;
                }
            }
        }
        "perf" => {
            if args.len() > 2 && args[2] == "recommend" {
                let diag = SystemDiagnostics::probe();
                let rec = PerformanceProfile::recommend(diag.total_memory_mib, diag.cpu_cores);
                println!("Recommended for this device: {:?}", rec.preset);
                println!(
                    "RAM: {} MiB | Scale: {:.2} | Chunks: {} | FPS: {}",
                    rec.memory_limit_mib, rec.render_scale, rec.render_distance, rec.target_fps
                );
            } else if args.len() > 2 && args[2] == "extreme" {
                let ext = PerformanceProfile::extreme();
                println!("Extreme Flagship Profile: {:?}", ext.preset);
                println!(
                    "RAM: {} MiB | Scale: {:.2} | Chunks: {} | FPS: {}",
                    ext.memory_limit_mib, ext.render_scale, ext.render_distance, ext.target_fps
                );
            } else if args.len() >= 7 && args[2] == "custom" {
                let ram: u32 = args[3].parse().unwrap_or(1024);
                let scale: f32 = args[4].parse().unwrap_or(0.75);
                let chunks: u32 = args[5].parse().unwrap_or(8);
                let fps: u32 = args[6].parse().unwrap_or(60);
                match PerformanceProfile::custom(ram, scale, chunks, fps) {
                    Ok(p) => {
                        println!("Custom configuration valid: {:?}", p.preset);
                        println!(
                            "RAM: {} MiB | Scale: {:.2} | Chunks: {} | FPS: {}",
                            p.memory_limit_mib, p.render_scale, p.render_distance, p.target_fps
                        );
                    }
                    Err(e) => {
                        eprintln!("Invalid custom performance parameters: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                eprintln!("Usage: horizon-cli perf <recommend | extreme | custom <RAM> <SCALE> <CHUNKS> <FPS>>");
                return ExitCode::FAILURE;
            }
        }
        "touch" => {
            if args.len() < 5 || args[2] != "export" {
                eprintln!("Usage: horizon-cli touch export <default|compact|pojav> <OUTPUT_PATH>");
                return ExitCode::FAILURE;
            }
            let layout = match args[3].as_str() {
                "compact" => TouchLayout::compact_low_end(),
                "pojav" => TouchLayout::pojav_touchpad(),
                _ => TouchLayout::default_phone(),
            };
            let out_path = PathBuf::from(&args[4]);
            if let Err(e) = layout.save_to_file(&out_path) {
                eprintln!("Failed to save touch layout to {:?}: {}", out_path, e);
                return ExitCode::FAILURE;
            }
            println!("Saved `{}` layout to {:?}", layout.name, out_path);
        }
        "content" => {
            if args.len() < 4 {
                eprintln!("Usage: horizon-cli content <worlds|mods|toggle> <PROFILE_ID> [ARGS]");
                return ExitCode::FAILURE;
            }
            let profiles = launcher.profiles().list_profiles().unwrap_or_default();
            let profile = match profiles.into_iter().find(|p| p.id == args[3]) {
                Some(p) => p,
                None => {
                    eprintln!("Profile `{}` not found.", args[3]);
                    return ExitCode::FAILURE;
                }
            };

            match args[2].as_str() {
                "worlds" => {
                    let worlds = list_worlds(&profile.game_dir).unwrap_or_default();
                    println!("Worlds in `{}` ({}):", profile.name, worlds.len());
                    for w in worlds {
                        println!("  - {} ({} bytes)", w.display_name, w.size_bytes);
                    }
                }
                "mods" => {
                    let mods = list_mods(&profile.game_dir).unwrap_or_default();
                    println!("Mods in `{}` ({}):", profile.name, mods.len());
                    for m in mods {
                        let status = if m.enabled { "ENABLED" } else { "DISABLED" };
                        println!("  - [{}] {} ({} bytes)", status, m.filename, m.size_bytes);
                    }
                }
                "toggle" => {
                    if args.len() < 6 {
                        eprintln!("Usage: horizon-cli content toggle <PROFILE_ID> <MOD_FILENAME> <on|off>");
                        return ExitCode::FAILURE;
                    }
                    let enable = args[5].eq_ignore_ascii_case("on");
                    match toggle_mod(&profile.game_dir, &args[4], enable) {
                        Ok(new_p) => println!(
                            "Toggled mod to: {:?}",
                            new_p.file_name().unwrap_or_default()
                        ),
                        Err(e) => {
                            eprintln!("Error toggling mod: {e}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                other => {
                    eprintln!("Unknown content command: {other}");
                    return ExitCode::FAILURE;
                }
            }
        }
        unknown => {
            eprintln!("Unknown command: `{unknown}`. Run with `--help` for available commands.");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}
