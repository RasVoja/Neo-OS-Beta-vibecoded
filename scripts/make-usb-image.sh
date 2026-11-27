#!/bin/sh
# Builds a bootable Neo Boing Platinum OS Lite image for x86_64 PCs:
#   scripts/make-usb-image.sh [default-entry]
# The image is a hybrid ISO made by grub-mkrescue: it boots from a CD or,
# written to a USB stick (dd, or Raspberry Pi Imager / balenaEtcher), on
# PCs with BIOS or UEFI. GRUB loads Fortress with Multiboot2 and the boot
# image as its first module. Prints the image's path.
#
# Menu entries: 0 Neo Boing Platinum OS Lite (default), 1 the boot self-test.
# Needs grub-pc-bin, grub-efi-amd64-bin, xorriso and mtools (Debian/Ubuntu).
set -e
cd "$(dirname "$0")/.."
default=${1:-0}
cargo build --release --quiet
bootfs=$(./scripts/make-bootfs.sh x86_64)
stage=target/usb-x86_64
rm -rf "$stage"
mkdir -p "$stage/boot/grub"
cp target/x86_64-unknown-none/release/fortress "$stage/boot/fortress"
cp "$bootfs" "$stage/boot/bootfs.tar"
cat > "$stage/boot/grub/grub.cfg" <<CFG
# Output goes to the screen and to the first serial port.
serial --unit=0 --speed=115200
terminal_input console serial
terminal_output console serial
# Video drivers, so GRUB can set the screen mode Fortress asks for.
insmod all_video
set timeout=3
set default=$default

menuentry "Neo Boing Platinum OS Lite" {
    multiboot2 /boot/fortress neo.shell
    module2 /boot/bootfs.tar bootfs
    boot
}

menuentry "Neo Boing Platinum OS Lite: boot self-test" {
    multiboot2 /boot/fortress neo.test
    module2 /boot/bootfs.tar bootfs
    boot
}
CFG
image=target/neo-lite-x86_64.iso
grub-mkrescue -o "$image" "$stage" >/dev/null 2>&1 || { grub-mkrescue -o "$image" "$stage"; exit 1; }
echo "$image"
