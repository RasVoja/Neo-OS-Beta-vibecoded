#!/bin/sh
# Build Fortress and boot it in QEMU (x86_64). Serial output goes to this terminal.
# Quit QEMU with Ctrl-A then X. Extra arguments go to the kernel command line.
set -e
cd "$(dirname "$0")/.."
cargo build --release
exec qemu-system-x86_64 -machine q35 -m 512M -nographic -no-reboot \
    -kernel target/x86_64-unknown-none/release/fortress -append "$*"
