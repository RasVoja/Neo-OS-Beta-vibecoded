# Neo Boing Platinum OS: stanje i ciljevi

[English](STATUS.md) · **Srpski** · Ažurirano 27. novembra 2026.

Ova strana prati šta je urađeno, a šta planirano, oblast po oblast. Takođe određuje **prag za doprinose**: trenutak kada se kod seli iz privatnog radnog repozitorijuma u ovaj, da bi drugi mogli da ga izgrade, pokrenu i dopune. Prag je dostignut 18. novembra 2026. i razvoj se nastavlja ovde.

## Oznake

Svaka stavka ispod i u [FEATURES.sr.md](FEATURES.sr.md) nosi jednu oznaku: **[gotovo]** radi i proverava se u CI-ju · **[u izradi]** gradi se sada · **[lite]** planirano za Neo Boing Platinum OS Lite na običnom x86_64 i ARM-u, može odmah u plan · **[koncept]** traži Neo hardver koji ne postoji (konceptni hardver). Sve što ne traži Neo hardver je [lite]. (U engleskoj verziji: [done], [next], [lite], [concept].)

## Prekretnice

| Prekretnica | Značenje | Stanje |
| --- | --- | --- |
| **M0: seme jezgra** | Malo jezgro koje se podiže, pokreće izolovane programe u Rustu i prolazi automatski test podizanja | [gotovo] |
| **M1: otvoreno za doprinose** | Kod se seli ovde. Dve arhitekture, programi sa diska, tastatura, ekran, osnovna komandna linija i prvi prozor, uz uputstvo za saradnike i početne zadatke | [gotovo], 22 od 22 cilja (18. novembar 2026.) |
| **M2: Neo Boing Platinum OS Lite alfa** | Podiže se sa USB-a na pravom PC-ju (Neo Revive), mreža, prozori Neo Kit-a, menadžer fajlova, prve svakodnevne aplikacije, Linux kutija sa Flatpakom (prve igre, Album, FilmCraft, PhotoCraft, qBittorrent) | [u izradi]: rad na podizanju na pravom PC-ju je počeo |
| **M3: Neo Boing Platinum OS Lite beta** | Ažuriranja (Obnova), privatnost (Brava), rezervne kopije (Čuvar), katalog Riznica, program Pomoć sa F1, srpski i engleski kompletni | [lite] |
| **AmigaOS mogućnosti** | Sloj AROS API-ja, ljuska i C: naredbe, Zune, Wanderer, Directory Opus 5, FS-UAE i Janus-UAE, oko 106 čovek-meseci neto; vidi [AMIGA.sr.md](AMIGA.sr.md) | [lite]: FS-UAE i naredbe ljuske uz M2, ostalo posle M3 |
| **Klasični Mac izgled i programi** | Klasični prozori i kontrole za temu „Prvi Sto”, klasični Mac emulatori u Arkadi, procena ponovo napisanog System 7 (Kelsidavis/System7, MIT); oko 13 čovek-meseci; vidi [CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md) | [lite]: tema uz M3, ostalo posle M3 |
| **Igre, kreativni i muzički programi u Lite-u** | Gotove igre preko Flatpaka, Arkada (MAME, DOSBox, ScummVM, RetroArch), naše Neo igre; FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), Album (gThumb); HippoPlayer, qBittorrent, Hydrogen, treker; vidi [GAMES.sr.md](GAMES.sr.md), [CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md), [MUSIC.sr.md](MUSIC.sr.md) | [lite]: Flatpak programi uz M2, Neo Kit verzije posle radne površine |
| **Konceptni računar** | Neo 500, 2000, 4000, Neo 100, Neo paket čipova, FPGA jezgra; vidi [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md) | [koncept]: ne koči sistem |

## Ciljevi za M1 po oblastima

### Jezgro (Tvrđava) · 5 od 6

- [x] Podiže se na x86_64 u QEMU-u, sa serijskom konzolom i testom podizanja u CI-ju
- [x] Izuzeci procesora, sopstvene tabele stranica jezgra, zaštitna stranica steka
- [x] Upravljanje celom fizičkom memorijom, hip jezgra
- [x] Tajmer, raspoređivač sa prekidanjem, uspavljivanje i parkiranje niti
- [x] Podiže se na AArch64 (QEMU `virt`) uz isti test podizanja: tabele stranica, tajmer, raspoređivač i portovi za poruke rade i tamo
- [x] Zapisana odluka: kada privremeno jezgro ustupa mesto Genode-u na seL4: prihvaćena 2026-11-15, uz probu na Genode-u pokrenutu odmah. Prvi Neo program već radi neizmenjen kao Genode komponenta na seL4 pod QEMU-om (2026-11-18)

### Programi i poruke (seme Exec-N) · 5 od 5

- [x] Programi u korisničkom režimu, svaki u svom adresnom prostoru, sa sistemskim pozivima; pad zaustavlja samo taj program
- [x] Imenovani portovi za poruke i ovlašćenja (pravo slanja i prijema, dozvola pretrage)
- [x] Rust okruženje za programe, ELF učitavač
- [x] Programi se učitavaju sa slike diska umesto da budu ugrađeni u jezgro (slika za podizanje i virtio disk)
- [x] Prstenovi deljene memorije za velike poruke: prsten bez zaključavanja u deljenom bloku, sa portom kao zvoncem; samoprovera šalje megabajt kroz prsten od 64 KiB na obe arhitekture

### Drajveri i servisi · 4 od 4

- [x] Servis konzole u korisničkom režimu: jedini program koji dira serijski port
- [x] Unos sa tastature: PS/2 drajver u korisničkom režimu na x86_64, virtio-input na AArch64; američki i srpski latinični raspored (menja se sa Alt+Shift ili klikom na Traku, koja pokazuje US ili SR), pa se č, ć, ž, š, đ kucaju u Neo DOS-u i njegovom prozoru
- [x] Jednostavan servis za ekran (pikseli na ekranu): servis u korisničkom režimu na x86_64 (VGA) i AArch64 (ramfb), isti program na obe
- [x] Blok uređaj (virtio) i sistem datoteka samo za čitanje

### Komandna linija i radna površina · 2 od 2

- [x] Neo DOS, osnovni: izlistaj, prikaži i pokreni fajlove (Neo DOS 0.2, sa volumenima BOOT: i DH0:, na obe arhitekture; red komande se uređuje strelicama, tasterima Home, End i Delete, a strelice gore i dole vraćaju ranije komande)
- [x] Prvi prozor na ekranu i pločica Sat (plan radne površine D0 do D3): prozor je nacrtan i vuče se mišem, a Sat otkucava, na obe arhitekture

### Spremno za saradnike · 2 od 5

- [x] Test podizanja jednom komandom, koji CI pokreće na svaku izmenu
- [x] Dnevnik napretka korak po korak
- [x] Uputstvo za saradnike: izgradnja, stil koda, kako se dodaje program ili servis (na srpskom i engleskom; seli se ovde zajedno sa kodom)
- [x] Izabrana licenca za kod: GNU GPL verzija 3 ili kasnija, za ovaj i za razvojni repozitorijum
- [x] Najmanje deset početnih zadataka kao issues, svaki sa jasnim testom: dvanaest, issue-i #3 do #14

## Posle M1

- **Put ka hardveru [u izradi]:** UEFI podizanje na pravim PC računarima, zatim Raspberry Pi 4/5 [lite], pa Neo 100 [koncept].
  - [x] Slika za podizanje na PC-ju (ISO za CD ili USB fleš): GRUB učitava Tvrđavu preko Multiboot2, na BIOS-u i UEFI-ju; testirano u CI-ju sa SeaBIOS-om i OVMF-om
  - [x] Radna površina na ekranu koji podesi učitavač (UEFI GOP ili VBE), testirano i bez ikakve VGA kartice
  - [x] USB tastatura i miš (xHCI kontroler, postavljanje uređaja, HID boot izveštaji, prekidi preko MSI ili MSI-X)
  - [x] Priprema za pravi PC: svi PCI busovi i visoki BAR-ovi, USB hubovi, ACPI tabele (procesori, rutiranje prekida, gašenje)
  - [x] Spremno za probe: USB uređaji se utiču i izvlače u radu (portovi na računaru i iza razdelnika), poruke pri pokretanju na ekranu (`BOOTLOG` u Neo DOS-u), vodič za probu i šablon za izveštaj ([REAL-PC-TEST.sr.md](REAL-PC-TEST.sr.md))
  - [ ] Prvo podizanje na pravim PC računarima
- **Genode na seL4 (odluka 0001) [u izradi]:**
  - [x] Neizmenjen Neo program radi kao Genode komponenta
  - [x] Portovi i poruke između Neo programa (server neo_ports)
  - [ ] Buđenje signalom, deljena memorija, ručke u porukama; zatim Neo DOS
- **Radna površina [u izradi]:** [gotovo] Dijana, logo i prva stanarka, živi na Traci (pozdravlja, njuška, spava, odgovara na klik, daje savet posle greške, kaže laku noć pri gašenju); sledi kompozitor, pločice (Sat, Kalkulator, Merač), Polica i Traka, prevlačenje, Neo meni i paleta komandi, pa izgled Neo Srebro.
- **Fajlovi [lite]:** Ormar Pro (dva panela, pravila Magelan), Zgusni (formati kompresije), Btrfs i EROFS.
- **Svakodnevne aplikacije [lite]:** pregledač, kancelarija, pošta, Album (gThumb), Gramofon sa HippoPlayer-om, qBittorrent, prvo preko zrelih otvorenih projekata u Linux kutiji.
- **Igre [lite]:** SuperTuxKart, 0 A.D., Luanti, Frozen Bubble, Battle for Wesnoth preko Flatpaka; Arkada sa MAME-om, DOSBox-om, ScummVM-om, RetroArch-om; zatim naše Neo igre ([GAMES.sr.md](GAMES.sr.md)).
- **Pomoć [lite]:** korisnički vodič kao HTML i PDF već postoji u [guide/](../guide/html-sr/index.html) [gotovo]; program Pomoć, F1 i slike pravog ekrana dolaze sa radnom površinom ([HELP.sr.md](HELP.sr.md)).
- **Konceptni računar [koncept]:** Neo ploče i čipovi ostaju konceptni hardver; Lite umesto njih koristi softverske verzije ([CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md)).

## Ukupan napredak prema specifikaciji

Specifikacija procenjuje Neo Boing Platinum OS 1.0 na 5 350 čovek-meseci. Do sada urađeno kodiranje iznosi oko 3 100 inženjerskih sati, otprilike 19,4 čovek-meseci ili 0,36% te procene. To je seme jezgra i prva radna površina (sa Dijanom). Deo toga može biti zamenjen prelaskom na Genode na seL4, pa je ovo gornja granica.
