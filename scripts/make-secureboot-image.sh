#!/bin/sh
# Builds a USB disk image that boots with Secure Boot on (UEFI only):
#   scripts/make-secureboot-image.sh [default-entry]
# The Microsoft-signed shim starts our own loader (loader/, signed with the
# key in $NEO_SB_KEY_DIR, made on first use), which starts Fortress as GRUB
# does. The public key is on the stick as neo-secure-boot.der: enrol it once
# (the shim's blue "MOK manager" screen: Enroll key from disk). Prints the
# image's path. The default entry (neo.shell, neo.text or neo.test) is for tests.
# Needs shim-signed, sbsigntool, openssl and mtools (Debian/Ubuntu).
set -e
cd "$(dirname "$0")/.."
default=${1:-}
keys=${NEO_SB_KEY_DIR:-target/secure-boot}
mkdir -p "$keys"
if [ ! -f "$keys/neo-sb.key" ]; then
    openssl req -new -x509 -newkey rsa:2048 -sha256 -nodes -days 3650 \
        -subj "/CN=Neo Boing Platinum OS Lite Secure Boot key/" \
        -keyout "$keys/neo-sb.key" -out "$keys/neo-sb.crt" >/dev/null 2>&1
    openssl x509 -in "$keys/neo-sb.crt" -outform DER -out "$keys/neo-sb.der"
fi
cargo build --release --quiet
(cd loader && cargo build --release --quiet --target x86_64-unknown-uefi)
bootfs=$(./scripts/make-bootfs.sh x86_64)
stage=target/secureboot-x86_64
rm -rf "$stage"
mkdir -p "$stage"
sbsign --key "$keys/neo-sb.key" --cert "$keys/neo-sb.crt" \
    --output "$stage/grubx64.efi" loader/target/x86_64-unknown-uefi/release/neo-loader.efi >/dev/null
image=target/neo-lite-x86_64-secureboot.img
rm -f "$image"
truncate -s 96M "$image"
# One partition (type EF, EFI System) from 1 MiB to the end, in an MBR.
python3 - "$image" <<'PY'
import struct, sys
img = sys.argv[1]
sectors = 96 * 2048
start = 2048
entry = struct.pack("<B3sB3sII", 0x80, b"\xfe\xff\xff", 0xEF, b"\xfe\xff\xff", start, sectors - start)
mbr = bytearray(512)
mbr[446:446 + 16] = entry
mbr[510:512] = b"\x55\xaa"
with open(img, "r+b") as f:
    f.write(mbr)
PY
part="$image@@1048576"
mformat -i "$part" -F -v NEOBOOT ::
mmd -i "$part" ::/EFI ::/EFI/BOOT ::/boot
mcopy -i "$part" /usr/lib/shim/shimx64.efi.signed ::/EFI/BOOT/BOOTX64.EFI
mcopy -i "$part" /usr/lib/shim/mmx64.efi ::/EFI/BOOT/mmx64.efi
mcopy -i "$part" "$stage/grubx64.efi" ::/EFI/BOOT/grubx64.efi
mcopy -i "$part" target/x86_64-unknown-none/release/fortress ::/boot/fortress
mcopy -i "$part" "$bootfs" ::/boot/bootfs.tar
mcopy -i "$part" "$keys/neo-sb.der" ::/neo-secure-boot.der
if [ -n "$default" ]; then
    echo "$default" > "$stage/default.txt"
    mcopy -i "$part" "$stage/default.txt" ::/boot/default.txt
fi
echo "$image"
