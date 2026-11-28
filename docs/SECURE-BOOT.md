# Secure Boot (plan)

Status: **not supported yet.** Boot the stick with Secure Boot switched off
(BIOS: Secure Boot = Disabled, or "Other OS"). This page says what it takes.

## Why the stick does not boot with Secure Boot on

The firmware only starts programs signed by a key it trusts (Microsoft's
keys on nearly every PC). Our stick has an unsigned GRUB, and Fortress is an
ELF file loaded by GRUB with Multiboot2. A signed GRUB would not help: with
Secure Boot, GRUB asks the shim to check every kernel it loads, and the shim
can only check signed PE files, not an ELF.

## The way through (own signed loader, no GRUB)

1. **Shim**: ship the distribution's Microsoft-signed `shimx64.efi` as
   `EFI/BOOT/BOOTX64.EFI`, and `mmx64.efi` (MokManager) beside it.
2. **Neo loader**: a small UEFI program (Rust, target `x86_64-unknown-uefi`)
   named `grubx64.efi` (the name the shim starts next). It reads
   `boot/fortress` and the boot image from the stick, builds the Multiboot2
   information Fortress already understands (memory map, framebuffer from
   GOP, ACPI RSDP, command line), exits boot services and jumps in. It is a
   replacement for GRUB, and a menu of the three entries (desk, text only,
   self-test) is drawn by it.
3. **Signing**: the loader is signed with our own key (`sbsign`). The public
   certificate (`neo-secure-boot.der`) is on the stick; the tester enrols it
   once through MokManager (first boot: "Enrol key from disk"), after which
   Secure Boot can stay on. Only the loader needs signing, because it is the
   thing the shim checks; the loader may check Fortress's hash itself.
4. **Test**: QEMU with OVMF's Secure Boot variables
   (`OVMF_VARS_4M.ms.fd`), key enrolment through `virt-fw-vars`.

Work: about 4 steps (loader, menu and Multiboot2 info, signing and ISO
layout, test). Fortress needs no change except to accept the information
from the loader instead of from GRUB.

## What to do meanwhile

Switch Secure Boot off in the BIOS, or enrol nothing: the stick boots with
BIOS or UEFI with Secure Boot off (tested in QEMU, and on the real
Threadripper test PC).
