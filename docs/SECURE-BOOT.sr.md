# Secure Boot

Običan USB (GRUB, `neo-boing-platinum-os-lite-x86_64.iso`) **ne** diže se sa
uključenim Secure Boot-om: GRUB-ov shim ne može da proveri Fortress, koji je
ELF fajl, a ne potpisan PE. Za računare sa Secure Boot-om postoji druga slika:

    neo-boing-platinum-os-lite-x86_64-secureboot.img

Upiši je na USB kao sliku diska (Rufus u DD režimu, balenaEtcher ili `dd`).
Radi samo na UEFI računarima, sa uključenim ili isključenim Secure Boot-om.

## Prvo dizanje (jednom po računaru): upis ključa

1. Podigni sa USB-a. Sa uključenim Secure Boot-om shim još ne zna naš ključ i
   prikazuje plavi ekran, **Perform MOK management**.
2. Izaberi **Enroll key from disk**, pa USB, pa `neo-secure-boot.der`, pa
   **Continue**, **Yes** (upis), **Reboot**.
3. Podigni sa USB-a opet. Mali meni (1 desktop, 2 samo tekst, 3 self-test) sam
   pokreće desktop posle 3 sekunde.

Ključ ostaje u MOK listi računara; druge slike napravljene istim ključem
dižu se odmah. Potpisan je samo naš loader.

## Kako radi

- `EFI/BOOT/BOOTX64.EFI` je distribucijski **shim** potpisan od Microsofta.
- `EFI/BOOT/grubx64.efi` (ime koje shim pokreće sledeće) je **naš loader**
  (`loader/`, Rust, cilj `x86_64-unknown-uefi`), potpisan našim ključem
  (`sbsign`), sa SBAT zapisom koji shim traži.
- Loader čita `boot/fortress` i `boot/bootfs.tar`, pravi Multiboot2 podatke koje
  Fortress već razume (komandna linija, boot slika, mapa memorije, framebuffer,
  ACPI tabela), napušta UEFI i skače na 32-bitni ulaz Fortress-a. Fortress nije
  menjan (samo je tabela mape memorije veća).
- `scripts/make-secureboot-image.sh` pravi sliku; ključ se pravi pri prvom
  korišćenju u `target/secure-boot` (ili `$NEO_SB_KEY_DIR`). Privatni ključ
  čuvaj: novi ključ znači novi upis.
- `scripts/test-secureboot-x86_64.sh` diže sliku u QEMU-u sa OVMF Secure Boot-om
  i Microsoftovim ključevima: sa upisanim ključem self-test prolazi, bez njega
  Fortress se ne pokreće.

## Nije urađeno / ograničenja

- Fortress nije potpisan ni merenje; proverava se samo loader (hash Fortress-a
  još ne).
- Probano samo u QEMU-u. Računar može da pokaže drugačiji MOK ekran; ako upis
  ne uspe, isključi Secure Boot u BIOS-u i koristi običan ISO.
- Samo UEFI; BIOS računari nemaju Secure Boot.
