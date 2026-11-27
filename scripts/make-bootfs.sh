#!/bin/sh
# Builds the boot image (a ustar archive) for one architecture:
#   scripts/make-bootfs.sh x86_64|aarch64 [disk]
# It holds the user programs under bin/ and is passed to QEMU with -initrd.
# With "disk", it builds the same archive as a disk image for DH0: instead,
# with its own README. Prints the path of the image.
set -e
cd "$(dirname "$0")/.."
arch=${1:?usage: make-bootfs.sh x86_64|aarch64 [disk]}
kind=${2:-boot}
triple="$arch-unknown-none"
root=$(pwd)
(cd user && cargo build --release --quiet --target "$triple" --target-dir "$root/target/user")
stage="$root/target/$kind-$arch"
rm -rf "$stage"
mkdir -p "$stage/bin"
for prog in hello-rust crash runner neodos display hello-window terminal; do
    cp "$root/target/user/$triple/release/$prog" "$stage/bin/"
done
if [ "$arch" = x86_64 ]; then
    for prog in console hello-console keyboard; do
        cp "$root/target/user/$triple/release/$prog" "$stage/bin/"
    done
fi
if [ "$kind" = disk ]; then
    echo "Neo OS Lite disk DH0: ($arch)" > "$stage/README"
    image="$root/target/disk-$arch.img"
else
    echo "Neo OS Lite boot image ($arch)" > "$stage/README"
    image="$root/target/bootfs-$arch.tar"
fi
tar --format=ustar --owner=0 --group=0 --numeric-owner -cf "$image" -C "$stage" README bin
echo "$image"
