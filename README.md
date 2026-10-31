# Horizon Mobile

A Rust-only Android host for the Horizon 26.3 native game.

This project is not a Pojav-style JVM launcher. The launcher, installer,
native game host, renderer bridge, and Android UI are Rust. The target is
Android arm64-v8a first, with low-memory defaults and no Java or Kotlin app
code.

## What is here now

- A pinned upstream submodule at upstream/horizon for Horizon's MIT-licensed
  Rust game source.
- A safe Rust core that owns isolated profiles, a fixed 26.3 install layout,
  native payload validation, atomic install records, and checksum-verified
  downloads.
- Native-only renderer choices: Auto, Zink, VirGL, GL4ES, and ANGLE.
- Tests for fixed 26.3 profiles and failed-download safety.

## Current state

This is the Android port foundation, not a playable APK yet. The upstream
client does not currently ship an Android build, so the next work is a Rust
Android platform layer for its wgpu window, touch input, audio, lifecycle, and
native library packaging. The project does not include Minecraft code, assets,
jars, or world data.

## Non-negotiable rules

- Horizon 26.3 only.
- No JVM, no Java runtime, and no Java or Kotlin application source.
- Do not commit Mojang assets, jars, decompiled source, or prepared resources.
- Require a legally owned Minecraft copy before importing any game data.
- Verify every downloaded native payload before it is registered.

## Layout

- crates/launcher-core: Rust-only install/profile/download policy.
- upstream/horizon: pinned upstream game source; never modify it directly.
- .github/workflows: Rust checks for this repository.

## Upstream

Horizon is an independent, unofficial Rust rewrite of Minecraft Java Edition.
This repository is an Android-port host and is not affiliated with Horizon,
Mojang, or Microsoft. See UPSTREAM.md for the exact boundary and attribution.

## Development

Install Rust 1.90 and an Android NDK, then run:

~~~sh
cargo test -p horizon-mobile-core
cargo clippy -p horizon-mobile-core -- -D warnings
~~~

The commands above test only this project's Rust core. They do not download or
process game files.
