#!/data/data/com.termux/files/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo "  Horizon 26.3 Mobile Launcher Build Script (0% Java)     "
echo "=========================================================="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    echo "[-] Error: 'cargo' is not found in PATH."
    echo "    Install Rust using: pkg install rust (Termux) or rustup (Linux)"
    exit 1
fi

echo "[*] Workspace: $ROOT_DIR"
echo "[*] Toolchain: $(rustc --version)"

echo "[+] Step 1: Formatting check..."
cargo fmt --all -- --check

echo "[+] Step 2: Running workspace tests..."
cargo test --workspace

echo "[+] Step 3: Compiling release binaries..."
cargo build --release --workspace

echo "[+] Step 4: Verifying compiled artifacts..."
ls -lh target/release/libhorizon_mobile_core.* 2>/dev/null || true
ls -lh target/release/libhorizon_launcher_android.* 2>/dev/null || true
ls -lh target/release/horizon-cli 2>/dev/null || true

echo ""
echo "=========================================================="
echo "  Build completed successfully!                           "
echo "  - Core library:   target/release/libhorizon_mobile_core"
echo "  - Android Host:   target/release/libhorizon_launcher_android"
echo "  - CLI Tool:       target/release/horizon-cli"
echo "=========================================================="
