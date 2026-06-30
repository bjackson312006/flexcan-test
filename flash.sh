#!/bin/bash
# Flash script for the flexcan-test firmware.
# On WSL it copies the ELF to Windows and flashes via the Windows probe-rs
# (works around WSL USB passthrough issues). On native Linux it flashes
# directly with the local probe-rs.
# You'll need probe-rs installed (on Windows for WSL, locally for native Linux).
#
# Run from anywhere inside the repo. Build first with:
#   cargo build           (debug, the default)
#   cargo build --release (release)
# Select the profile with PROFILE=release, or point at an arbitrary ELF
# with the BINARY env var.
#
# This script also works as a Cargo runner (see .cargo/config.toml), so
# `cargo run` / `cargo run --release` build and flash in one step. When
# invoked that way, Cargo passes the freshly built ELF path as the first
# argument, which takes priority over the PROFILE/BINARY resolution below.

set -e

# Detect whether we're running under WSL or native Linux.
if [ -n "$WSL_DISTRO_NAME" ] || grep -qiE "(microsoft|wsl)" /proc/version 2>/dev/null; then
    IS_WSL=1
    echo "Detected WSL."
else
    IS_WSL=0
    echo "Detected Native Linux."
fi

CHIP="MCXA256"
TARGET="thumbv8m.main-none-eabihf"
PROFILE="${PROFILE:-debug}"

# Resolve the repo root relative to this script so it can be run from anywhere.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Cargo's runner passes the built ELF path as $1; fall back to BINARY env var
# or the conventional target path otherwise.
BINARY="${1:-${BINARY:-$SCRIPT_DIR/target/$TARGET/$PROFILE/flexcan-test}}"

WINDOWS_TEMP="C:\\temp"
WSL_WINDOWS_TEMP="/mnt/c/temp"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build' first, or set the PROFILE/BINARY environment variable"
    exit 1
fi

if [ "$IS_WSL" -eq 1 ]; then
    # Copy binary to Windows temp directory
    echo "Copying binary to Windows..."
    mkdir -p "$WSL_WINDOWS_TEMP"
    cp "$BINARY" "$WSL_WINDOWS_TEMP/flexcan-test"

    # Flash using Windows probe-rs
    echo "Flashing via Windows probe-rs..."
    powershell.exe -c "probe-rs run --chip $CHIP $WINDOWS_TEMP\\flexcan-test"
else
    # Flash directly with the local probe-rs
    echo "Flashing via probe-rs..."
    probe-rs run --chip "$CHIP" "$BINARY"
fi

echo "Done!"
