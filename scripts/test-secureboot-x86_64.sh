#!/bin/sh
# Boot test of the Secure Boot image (scripts/make-secureboot-image.sh): OVMF
# with Secure Boot on and Microsoft's keys, the image's own key enrolled as a
# MOK; the shim must start our loader, and Fortress's boot self-test must
# pass (exit status 33). A second boot without the key enrolled must NOT start
# Fortress (the shim's MOK manager comes up instead).
# Needs, besides QEMU: ovmf, shim-signed, sbsigntool, mtools, openssl and
# `pip install virt-firmware`.
set -e
cd "$(dirname "$0")/.."
image=$(./scripts/make-secureboot-image.sh neo.test | tail -1)
log=$(mktemp)
vars=$(mktemp)
trap 'rm -f "$log" "$vars"' EXIT
ovmf=/usr/share/OVMF
fail() { echo "SECURE BOOT TEST FAILED: $1"; exit 1; }

run() { # $1 = seconds, rest = more options
    secs=$1
    shift
    set +e
    timeout "$secs" qemu-system-x86_64 -machine q35,smm=on -global driver=cfi.pflash01,property=secure,value=on \
        -m 512M -display none -serial file:"$log" -no-reboot \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 -device qemu-xhci -device usb-kbd \
        -drive if=pflash,format=raw,unit=0,readonly=on,file=$ovmf/OVMF_CODE_4M.secboot.fd \
        -drive if=pflash,format=raw,unit=1,file="$vars" -drive format=raw,file="$image" "$@" >/dev/null 2>&1
    status=$?
    set -e
}

# The key enrolled as a MOK: Fortress starts and the self-test passes.
virt-fw-vars -i $ovmf/OVMF_VARS_4M.ms.fd -o "$vars" --add-mok 11111111-2222-3333-4444-555555555555 \
    "${NEO_SB_KEY_DIR:-target/secure-boot}/neo-sb.crt" >/dev/null 2>&1
run 120
echo "--- Secure Boot on, key enrolled (exit $status)"
grep -a "Multiboot2\|Usable RAM\|Boot image\|Screen:\|Self-test: display" "$log" || true
[ "$status" -eq 33 ] || fail "key enrolled: exit status $status, expected 33"
grep -aq "Booted by a Multiboot2 loader." "$log" || fail "not booted through Multiboot2 information"
grep -aq "Self-test: display service ok" "$log" || fail "the desk was not drawn"
grep -aq "Boot image at" "$log" || fail "boot image not found"

# The key not enrolled: Fortress must not start.
cp $ovmf/OVMF_VARS_4M.ms.fd "$vars"
run 25
echo "--- Secure Boot on, key not enrolled (exit $status)"
grep -aq "Booted by a Multiboot2" "$log" && fail "Fortress started without the key enrolled"
echo "SECURE BOOT TEST PASSED"
