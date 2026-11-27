# Proba Neo Boing Platinum OS Lite na pravom računaru

[English](REAL-PC-TEST.md) · **Srpski** · 27. novembar 2026.

ISO se pali u QEMU-u posle svake izmene. Ono što QEMU ne može da pokaže je kako se ponaša na pravim mašinama: drugi USB kontroleri, drugi firmver, druge ACPI tabele. Ova stranica je za svakoga ko želi da proba i da nam javi šta se desilo. Traje desetak minuta i ništa ne upisuje na tvoje diskove.

## Šta ti treba

- Računar sa x86_64 procesorom (Intel ili AMD), BIOS ili UEFI.
- USB ključ od 64 MB ili više; sve na njemu biće obrisano.
- USB tastatura i miš (radi i PS/2).
- Telefon, da fotografišeš ekran.

## 1. Upiši sliku

Preuzmi `neo-boing-platinum-os-lite-x86_64.iso` (ili ga napravi: `./scripts/make-usb-image.sh`) i upiši ga na ključ programom balenaEtcher, Raspberry Pi Imager, Rufus u **DD režimu**, ili sa `dd if=… of=/dev/sdX bs=4M` na Linuksu.

## 2. Pokreni

1. **Windows sa BitLocker-om:** ako menjaš Secure Boot, Windows posle može da traži ključ za oporavak. Prvo nađi ključ (tvoj Microsoft nalog, „BitLocker recovery key").
2. Slika nije potpisana: **isključi Secure Boot** za probu, i uključi ga posle.
3. Izaberi ključ u jednokratnom meniju za pokretanje (često F8, F11, F12 ili Esc dok se računar pali).
4. GRUB pokazuje dve stavke. Prva, **Neo Boing Platinum OS Lite**, pokreće radnu površinu; druga pokreće samoproveru i korisna je kad prva ne pokaže ništa.

## 3. Šta treba da vidiš

- Radnu površinu: pozadinu, prozor Dobrodošli, pločicu Sat, prozor Neo DOS-a i Traku na dnu.
- **Dijanu** na Traci, desno od sredine, kako pozdravlja. Približi miš: njuška. Ne diraj miš pola minuta: zaspi.
- Kucanje ide u Neo DOS. Probaj `HELP`, `INFO`, `DIR`.
- Izvuci USB tastaturu i ponovo je utakni (ili u drugi port): treba da radi i dalje. Za sada se računaju samo portovi na samom računaru, ne portovi USB razdelnika (haba).

## 4. Pročitaj šta je jezgro našlo

Pravi računar nema serijsku liniju sa koje bi se čitale poruke pri pokretanju, pa ih Neo DOS pokazuje na ekranu:

| Komanda | Pokazuje |
| --- | --- |
| `BOOTLOG` | prvih 20 redova; `BOOTLOG 2`, `BOOTLOG 3` … sledeće strane |
| `BOOTLOG USB` | samo redove o USB-u: kontroler, njegove prekide (MSI, MSI-X ili prozivanje), svaki uređaj |
| `BOOTLOG ACPI` | procesore, rutiranje prekida, da li gašenje radi |
| `BOOTLOG PCI` | koliko je uređaja i magistrala nađeno |
| `BOOTLOG Screen` | koji frejmbafer koristi radna površina |

Fotografiši svaku stranu `BOOTLOG`-a, pa otkucaj `ENDCLI`: na računaru gde ACPI radi mašina se sama ugasi.

## 5. Javi nam

Otvori issue sa šablonom **Real PC report** (može i na srpskom). Dodaj:

- model računara ili matične ploče, procesor, BIOS ili UEFI;
- dokle je stiglo: GRUB meni, crn ekran, radna površina, kucanje;
- fotografije `BOOTLOG`-a;
- šta nije radilo: tastatura, miš, neki port, gašenje.

Svaki izveštaj, i „crn ekran posle GRUB-a", pomaže: kaže gde treba tražiti.

## Poznata ograničenja

- Još nema drajvera za mrežu, zvuk, diskove ni Wi-Fi; ništa se ne čuva.
- USB uređaji iza razdelnika se nađu pri pokretanju, ali utikanje u razdelnik kasnije se još ne vidi. USB 3 razdelnici se navedu, ali se ne obilaze; njihove tastature ipak rade preko USB 2 strane razdelnika.
- Ekran mora biti 800 x 600 sa 32 bita; GRUB to traži od firmvera. Ako firmver ne može, radna površina se ne pojavi (stavka za samoproveru to kaže).
