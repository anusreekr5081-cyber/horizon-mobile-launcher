# Horizon 26.3 Mobile

A 100% Rust Android Launcher and Native Host for Horizon 26.3.

**Zero Java, Zero Kotlin, Zero JVM.** Built from the ground up for low-end Android smartphones with Pojav/Mojo-launcher style customization, virtual touch controls, performance budgeting, and content management.

---

## Highlights

- **Pure Rust Native Architecture**: Built on `android-activity` (`ANativeActivity`) with `android:hasCode="false"`. No Java Virtual Machine, no Dalvik/ART overhead, and no JNI garbage collection pauses.
- **Horizon 26.3 Dedicated**: Locked strictly to the independent Horizon 26.3 Rust rewrite of Minecraft.
- **Low-End Phone Optimizations**:
  - Memory presets: **Ultra-Low (512 MiB)**, **Low (768 MiB)**, **Balanced (1024 MiB)**, **High (2048 MiB)**.
  - Viewport resolution scaler (0.50x, 0.65x, 0.75x, 1.0x) to eliminate GPU fillrate bottlenecks on Mali/Adreno budget chips.
  - Worker thread throttling to prevent thermal saturation on ARM big.LITTLE CPUs.
  - Forced 16-bit depth buffer and shader pipeline caching.
- **Pojav-Style Virtual Touch Controls**:
  - Full multitouch support with D-Pad, Analog Joystick, Jump, Sneak lock, Sprint lock, Attack/Mine, Use/Place, Inventory, Chat, Pause, F3, and F5 buttons.
  - Factory presets: **Default Phone** and **Compact Low-End** (battery & fillrate optimized).
  - JSON layout export and import.
- **Isolated Content & Add-on Management**:
  - Multi-profile management with independent `instances/<profile>/` directories.
  - World saves management (inspect, backup, delete).
  - Mod management (scan, toggle active/disabled).
  - Resource pack management.
- **Offline CLI Companion (`horizon-cli`)**:
  - Full terminal management directly in Termux or desktop Linux without needing Android Studio.

---

## Workspace Layout

```
├── .github/workflows/
│   ├── ci.yml               # Automated formatting, clippy, and unit tests
│   └── build-apk.yml        # Pure-native Android APK packaging & signing pipeline
├── android/
│   └── AndroidManifest.xml  # NativeActivity manifest (android:hasCode="false")
├── crates/
│   ├── launcher-core/       # Domain logic: profiles, install, touch, performance, content
│   ├── launcher-android/    # Android NativeActivity host, touch mapper & game bridge
│   └── launcher-cli/        # Standalone terminal manager (horizon-cli)
├── scripts/
│   └── build.sh             # One-step local build and verification script
├── upstream/
│   └── horizon/             # Pinned upstream Horizon 26.3 git submodule
├── Cargo.toml               # Cargo workspace manifest
├── rust-toolchain.toml      # Pinned Rust toolchain configuration
├── UPSTREAM.md              # Upstream boundary and licensing
└── README.md
```

---

## CLI Usage (`horizon-cli`)

The CLI runs directly on Termux (Android) or Linux:

```bash
# Display launcher status and active profiles
horizon-cli status

# Inspect hardware specs and get auto-detected performance recommendations
horizon-cli diagnostics

# Create a low-memory profile
horizon-cli profile create "LowEnd-Play" 512 virgl

# List configured profiles
horizon-cli profile list

# Export touch controls layout
horizon-cli touch export compact ./touch_layout.json

# View and toggle mods in a profile
horizon-cli content mods <PROFILE_ID>
horizon-cli content toggle <PROFILE_ID> <MOD_FILENAME> off
```

---

## Building & Verification

### Running Workspace Tests
```bash
cargo test --workspace
```

### Running Lints & Format Checks
```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

### Building Release Binaries
```bash
./scripts/build.sh
```

### Building the Android APK (Zero Java)
GitHub Actions automatically builds and signs the native `.apk` on every push to `master`. The workflow uses `cargo-ndk` to compile `libhorizon_launcher_android.so` for `arm64-v8a` and packages it into an APK using Android NDK/build-tools without Gradle.

---

## Legal & Upstream Boundary

- Horizon is an independent, unofficial Rust rewrite of Minecraft Java Edition.
- This repository is a native Android host and launcher; it is not affiliated with Horizon, Mojang, or Microsoft.
- **Zero Mojang code, jars, assets, or decompiled resources are committed or distributed here.**
- Players must possess a legally owned copy of the game. See [UPSTREAM.md](UPSTREAM.md) for full licensing details.
