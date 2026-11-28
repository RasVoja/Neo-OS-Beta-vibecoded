# Secure Boot

The normal stick (GRUB, `neo-boing-platinum-os-lite-x86_64.iso`) does **not**
start with Secure Boot on: GRUB's shim check cannot verify Fortress, which is an
ELF file, not a signed PE. For PCs with Secure Boot there is a second image:

    neo-boing-platinum-os-lite-x86_64-secureboot.img

Write it to a USB stick as a disk image (Rufus in DD mode, balenaEtcher, or
`dd`). It starts only on UEFI PCs, with Secure Boot on or off.

## First start (once per PC): enrol the key

1. Boot the stick. With Secure Boot on, the shim does not know our key yet and
   shows a blue screen, **Perform MOK management**.
2. Choose **Enroll key from disk**, then the stick, then
   `neo-secure-boot.der`, then **Continue**, **Yes** (enrol), **Reboot**.
   (Some boards ask for a MOK password only if one was set; there is none.)
3. Boot the stick again. A small menu (1 desk, 2 text only, 3 self-test)
   starts the desk after 3 seconds.

The key stays in the PC's own MOK list; other stick images made with the same
key start straight away. Only our loader is signed by it.

## How it works

- `EFI/BOOT/BOOTX64.EFI` is the distribution's Microsoft-signed **shim**.
- `EFI/BOOT/grubx64.efi` (the name the shim starts next) is **our loader**
  (`loader/`, Rust, target `x86_64-unknown-uefi`), signed with our key
  (`sbsign`) and carrying an SBAT record, which the shim demands.
- The loader reads `boot/fortress` and `boot/bootfs.tar`, builds the
  Multiboot2 information Fortress already understands (command line, boot
  image, memory map, framebuffer from the graphics output, ACPI table), leaves
  UEFI and jumps to Fortress's 32-bit entry through a small trampoline. Fortress
  is not changed (only its memory-map table got bigger, because UEFI maps have
  many entries).
- `scripts/make-secureboot-image.sh` makes the image. The key is made on first
  use in `target/secure-boot` (or `$NEO_SB_KEY_DIR`); keep the private key
  safe: a new key means enrolling again.
- `scripts/test-secureboot-x86_64.sh` boots the image in QEMU with OVMF's
  Secure Boot on and Microsoft's keys: with our key enrolled as a MOK, the
  boot self-test passes; without it, Fortress does not start.

## Not done / limits

- Fortress itself is not signed or measured; only the loader is checked.
  The loader does not check Fortress's hash yet.
- Tested in QEMU only. A PC may show a different MOK screen; if enrolling
  fails, switch Secure Boot off in the BIOS and use the normal ISO.
- UEFI only; BIOS PCs have no Secure Boot.
