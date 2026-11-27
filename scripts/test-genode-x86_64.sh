#!/bin/sh
# Runs the Genode trial (decision 0001): builds the neo-rt program in
# genode/neo-hello, then boots Genode on seL4 under QEMU with it.
#
# Needs a Genode 24.11 tree with the seL4 and grub2 ports prepared and an
# x86_64 build directory whose etc/build.conf has
#   KERNEL = sel4, BOARD = pc and
#   REPOSITORIES += <this repository>/genode/repos/neo
# (see genode/README.md). Not run in CI: the Genode toolchain is large.
#
# Usage: GENODE_DIR=/path/to/genode scripts/test-genode-x86_64.sh
set -e
: "${GENODE_DIR:?set GENODE_DIR to a Genode 24.11 tree}"
here=$(cd "$(dirname "$0")/.." && pwd)
(cd "$here/genode/neo-hello" && cargo build --release)
cd "$GENODE_DIR/build/x86_64"
make run/neo_hello KERNEL=sel4 BOARD=pc
echo "Genode trial: OK"
