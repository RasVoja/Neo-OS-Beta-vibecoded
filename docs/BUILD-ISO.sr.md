# Pokretačka slika, optimizovan build i instalacija pored Windowsa 11

[English](BUILD-ISO.md) · **Srpski** · Plan, 27. novembar 2026.

## 1. Šta već postoji

`scripts/make-usb-image.sh` pravi `target/neo-lite-x86_64.iso`: hibridni ISO koji se pokreće sa CD-a ili USB ključa na računarima sa BIOS-om ili UEFI-jem. GRUB učitava Fortress preko Multiboot2 i boot sliku kao modul. CI ga pokreće sa SeaBIOS-om i OVMF-om (`scripts/test-usb-x86_64.sh`). Treba mu `grub-pc-bin`, `grub-efi-amd64-bin`, `xorriso` i `mtools`.

Slika je živ sistem: ništa se ne upisuje ni na jedan disk. Instalera još nema, a fajl sistem je samo za čitanje.

## 2. Pravljenje i provera

```sh
./scripts/test-boot-x86_64.sh     # gradi sve, pali 11 puta
./scripts/make-usb-image.sh       # ISO
./scripts/test-usb-x86_64.sh      # pali ISO sa BIOS-om i sa UEFI-jem
```

Na USB ključ se upisuje sa `dd`, balenaEtcher ili Raspberry Pi Imager.

## 3. Optimizovan build

Cargo.toml gradi release sa `opt-level = "s"` (mali). Profil za brzinu je u `scripts/profiles/speed.env`: opt-level 3, potpuni LTO, jedna jedinica generisanja koda. **Nije probano**; učitaj ga, izgradi i pokreni probe paljenja pre nego što mu veruješ.

**Zastavice za određeni procesor još nisu bezbedne.** Cilj `x86_64-unknown-none` gradi se bez SSE i AVX. Zastavice poput `-C target-cpu=znver2` mogu da ih uključe, a jezgro još ne čuva i ne vraća to stanje. Vratiti se na to kad jezgro bude radilo sa FPU/SSE stanjem, i tek tada prvo za korisničke programe. Isto važi kasnije za Genode stranu.

## 4. Referentni računar

`scripts/profiles/ras-vojas.env` beleži vlasnikov računar: Threadripper PRO 3945WX (24 niti), ploča WRX80 Creator R2.0, 64 GB, Intel Arc B580, NVMe i disk od 16 TB, Windows 11, UEFI.

Šta se danas može očekivati na njemu:

| Deo | Očekivanje |
| --- | --- |
| Ekran | Preko UEFI GOP framebuffera, bez ubrzanja. Za to Arc B580 ne traži drajver |
| USB tastatura i miš | Zavisi na koji xHCI kontroler su vezani priključci. Ploča ih ima nekoliko; probati više portova |
| PCI i ACPI | Mnogo PCI magistrala; kod čita sve. Nije poznato kako se ponaša na ovoj ploči, zato proba |
| Mreža, zvuk, diskovi | Drajvera još nema. Kartice od 10G, Wi-Fi 6E i USB zvuk dolaze kasnije (M2 i dalje) |

## 5. Prvo paljenje na referentnom računaru

Vodič korak po korak za bilo koji računar, sa onim što treba pogledati i kako javiti, je [REAL-PC-TEST.sr.md](REAL-PC-TEST.sr.md). Neo DOS pokazuje poruke jezgra pri pokretanju komandom `BOOTLOG`.

Korak „Prvo paljenje na pravim PC-jevima” u [STATUS.sr.md](STATUS.sr.md) još je otvoren, a ovaj računar je prvi kandidat.

1. Napravi rezervnu kopiju onoga što je važno. Ovde ništa ne piše po diskovima, ali restart je restart.
2. **Prvo BitLocker.** Windows 11 Home može imati uključeno šifrovanje uređaja. Menjanje podešavanja Secure Boot može da zatraži ključ za oporavak. Nađi ključ (Microsoft nalog, „BitLocker recovery key”) ili suspenduj BitLocker pre nego što dirneš podešavanja firmvera.
3. Slika nije potpisana za Secure Boot. Isključi Secure Boot za probu, pokreni USB ključ iz jednokratnog boot menija, a posle ga ponovo uključi.
4. Zapiši ili slikaj šta ekran pokazuje i gde staje. Fotografiju i odgovore na pitanja iz odeljka 4 dodaj u `docs/PROGRESS.md`.
5. Ako je ekran crn, probaj drugu stavku menija (probu paljenja), drugi USB port i običan USB ključ upisan sa `dd`.

## 6. Instalacija pored Windowsa 11

**Pravila za svaku fazu:** nikad ne pisati po Windows particijama ni po Windows EFI sistemskoj particiji; koristiti samo slobodan prostor ili particiju napravljenu za Neo; pokazati plan i pitati pre upisa; poništavanje uklanja samo ono što je instaler dodao.

| Faza | Šta | Stanje |
| --- | --- | --- |
| 0 | Živ USB ili CD, ništa ne upisuje | radi |
| 1 | **Ručni dual boot, bez novog koda.** U Windowsu smanji jedan podatkovni volumen (vlasnik ima veliki drugi volumen na NVMe-u), napravi malu FAT32 particiju za Neo, kopiraj stablo `boot/` sa ISO-a tamo uz GRUB, dodaj stavku za paljenje u firmveru. Kako se stavka dodaje zavisi od firmvera; treba probati na referentnom računaru i ovde zapisati | za uraditi |
| 2 | **Instaler** u Neo-u: nabraja diskove, nudi samo slobodan prostor, traži da se ukuca ime diska za potvrdu, upisuje A/B sliku i GRUB na sopstvenu particiju, dodaje stavku za paljenje, nudi poništavanje | blokirano: traži NVMe i AHCI drajvere i fajl sistem za pisanje |
| 3 | Ažuriranja preko Obnove u drugu polovinu A/B slike | posle instalera |

Raspberry Pi 5 (i Pi 4) dolazi posle puta za PC.
