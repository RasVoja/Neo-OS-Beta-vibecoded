#!/bin/sh
# Boot test of the PC image (scripts/make-usb-image.sh): GRUB loads Fortress
# with Multiboot2, once with BIOS (SeaBIOS) and once with UEFI (OVMF), and
# the boot self-test must pass (exit status 33) both times.
# Needs, besides QEMU: grub-pc-bin, grub-efi-amd64-bin, xorriso, mtools, ovmf.
set -e
cd "$(dirname "$0")/.."
image=$(./scripts/make-usb-image.sh 1)
log=$(mktemp)
vars=$(mktemp)
trap 'rm -f "$log" "$vars"' EXIT
code=/usr/share/OVMF/OVMF_CODE_4M.fd
[ -f "$code" ] || code=/usr/share/OVMF/OVMF_CODE.fd
template=$(dirname "$code")/$(basename "$code" | sed 's/CODE/VARS/')
fail() { echo "USB IMAGE TEST FAILED: $1"; exit 1; }

run() { # $1 = name, rest = firmware options
    name=$1
    shift
    set +e
    timeout 120 qemu-system-x86_64 -machine q35 -m 512M -display none -serial stdio -no-reboot \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 "$@" -cdrom "$image" >"$log" 2>&1
    status=$?
    set -e
    echo "--- $name (exit $status)"
    grep -a "Multiboot2\|Usable RAM\|Boot image\|Self-test" "$log" || true
    [ "$status" -eq 33 ] || fail "$name: exit status $status, expected 33"
    grep -aq "Booted by a Multiboot2 loader." "$log" || fail "$name: not booted through Multiboot2"
    grep -aq "Boot image at" "$log" || fail "$name: boot image (GRUB module) not found"
}

run BIOS
cp "$template" "$vars"
run UEFI -drive if=pflash,format=raw,readonly=on,file="$code" -drive if=pflash,format=raw,file="$vars"
echo "USB IMAGE TEST PASSED"
