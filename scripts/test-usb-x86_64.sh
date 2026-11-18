#!/bin/sh
# Boot test of the PC image (scripts/make-usb-image.sh): GRUB loads Fortress
# with Multiboot2, once with BIOS (SeaBIOS) and once with UEFI (OVMF), and
# the boot self-test must pass (exit status 33) both times. A third boot
# uses UEFI with no VGA card at all (only ramfb behind UEFI GOP), so the
# desk must come up on the framebuffer GRUB hands over, as on a real PC.
# Each boot has a USB controller, which UEFI firmware may place above 4 GiB.
# Needs, besides QEMU: grub-pc-bin, grub-efi-amd64-bin, xorriso, mtools, ovmf.
set -e
cd "$(dirname "$0")/.."
image=$(./scripts/make-usb-image.sh 1)
log=$(mktemp)
vars=$(mktemp)
trap 'cp "$log" /tmp/claude-0/last-usb.log; rm -f "$log" "$vars"' EXIT
code=/usr/share/OVMF/OVMF_CODE_4M.fd
[ -f "$code" ] || code=/usr/share/OVMF/OVMF_CODE.fd
template=$(dirname "$code")/$(basename "$code" | sed 's/CODE/VARS/')
fail() { echo "USB IMAGE TEST FAILED: $1"; exit 1; }

run() { # $1 = name, rest = firmware options
    name=$1
    shift
    set +e
    timeout 120 qemu-system-x86_64 -machine q35 -m 512M -display none -serial stdio -no-reboot \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 -device qemu-xhci -device usb-kbd "$@" -cdrom "$image" >"$log" 2>&1
    status=$?
    set -e
    echo "--- $name (exit $status)"
    grep -a "Multiboot2\|Usable RAM\|Boot image\|Self-test" "$log" || true
    [ "$status" -eq 33 ] || fail "$name: exit status $status, expected 33"
    grep -aq "Booted by a Multiboot2 loader." "$log" || fail "$name: not booted through Multiboot2"
    grep -aq "Boot image at" "$log" || fail "$name: boot image (GRUB module) not found"
    grep -aq "Screen: boot loader's framebuffer" "$log" || fail "$name: the loader's framebuffer was not used"
    grep -aq "Self-test: display service ok" "$log" || fail "$name: the desk was not drawn"
    grep -a "PCI:\|USB controller at\|USB: xHCI" "$log" || true
    grep -aq "USB: xHCI" "$log" || fail "$name: the USB controller did not start"
    grep -a "^ACPI" "$log" || true
    grep -aq "power off ready" "$log" || fail "$name: ACPI tables not read"
}

run BIOS
cp "$template" "$vars"
run UEFI -drive if=pflash,format=raw,readonly=on,file="$code" -drive if=pflash,format=raw,file="$vars"
cp "$template" "$vars"
run "UEFI, GOP only" -vga none -device ramfb -drive if=pflash,format=raw,readonly=on,file="$code" -drive if=pflash,format=raw,file="$vars"
echo "USB IMAGE TEST PASSED"
