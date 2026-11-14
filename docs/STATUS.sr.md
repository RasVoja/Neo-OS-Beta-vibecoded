# NeoOS: stanje i ciljevi

[English](STATUS.md) · **Srpski** · Ažurirano 14. novembra 2026.

Ova strana prati šta je urađeno, a šta planirano, oblast po oblast. Takođe određuje **prag za doprinose**: trenutak kada se kod seli iz privatnog radnog repozitorijuma u ovaj, da bi drugi mogli da ga izgrade, pokrenu i dopune.

## Prekretnice

| Prekretnica | Značenje | Stanje |
| --- | --- | --- |
| **M0: seme jezgra** | Malo jezgro koje se podiže, pokreće izolovane programe u Rustu i prolazi automatski test podizanja | Urađeno |
| **M1: otvoreno za doprinose** | Kod se seli ovde. Dve arhitekture, programi sa diska, tastatura, ekran, osnovna komandna linija i prvi prozor, uz uputstvo za saradnike i početne zadatke | 14 od 22 cilja (64%) |
| **M2: Neo OS Lite alfa** | Podiže se sa USB-a na pravom PC-ju (Neo Revive), mreža, prozori Neo Kit-a, menadžer fajlova, prve svakodnevne aplikacije | U planu |
| **M3: Neo OS Lite beta** | Ažuriranja (Obnova), privatnost (Brava), rezervne kopije (Čuvar), katalog Riznica, srpski i engleski kompletni | U planu |

## Ciljevi za M1 po oblastima

### Jezgro (Tvrđava) · 5 od 6

- [x] Podiže se na x86_64 u QEMU-u, sa serijskom konzolom i testom podizanja u CI-ju
- [x] Izuzeci procesora, sopstvene tabele stranica jezgra, zaštitna stranica steka
- [x] Upravljanje celom fizičkom memorijom, hip jezgra
- [x] Tajmer, raspoređivač sa prekidanjem, uspavljivanje i parkiranje niti
- [x] Podiže se na AArch64 (QEMU `virt`) uz isti test podizanja: tabele stranica, tajmer, raspoređivač i portovi za poruke rade i tamo
- [ ] Zapisana odluka: kada privremeno jezgro ustupa mesto Genode-u na seL4

### Programi i poruke (seme Exec-N) · 4 od 5

- [x] Programi u korisničkom režimu, svaki u svom adresnom prostoru, sa sistemskim pozivima; pad zaustavlja samo taj program
- [x] Imenovani portovi za poruke i ovlašćenja (pravo slanja i prijema, dozvola pretrage)
- [x] Rust okruženje za programe, ELF učitavač
- [x] Programi se učitavaju sa slike diska umesto da budu ugrađeni u jezgro (slika za podizanje i virtio disk)
- [ ] Prstenovi deljene memorije za velike poruke

### Drajveri i servisi · 2 od 4

- [x] Servis konzole u korisničkom režimu: jedini program koji dira serijski port
- [ ] Servis za tastaturu
- [ ] Jednostavan servis za ekran (pikseli na ekranu)
- [x] Blok uređaj (virtio) i sistem datoteka samo za čitanje

### Komandna linija i radna površina · 1 od 2

- [x] Neo DOS, osnovni: izlistaj, prikaži i pokreni fajlove (Neo DOS 0.2, sa volumenima BOOT: i DH0:, na obe arhitekture)
- [ ] Prvi prozor na ekranu i pločica Sat (plan radne površine D0 do D3)

### Spremno za saradnike · 2 od 5

- [x] Test podizanja jednom komandom, koji CI pokreće na svaku izmenu
- [x] Dnevnik napretka korak po korak
- [ ] Uputstvo za saradnike: izgradnja, stil koda, kako se dodaje program ili servis
- [ ] Izabrana licenca za kod
- [ ] Najmanje deset početnih zadataka kao issues, svaki sa jasnim testom

## Posle M1

- **Put ka hardveru:** UEFI podizanje na pravim PC računarima, zatim Raspberry Pi 4/5, pa Neo 100.
- **Radna površina:** kompozitor, pločice (Sat, Kalkulator, Merač), Polica i Traka, prevlačenje, Neo meni i paleta komandi, pa izgled Neo Srebro.
- **Fajlovi:** menadžer fajlova sa dva panela, formati kompresije, Btrfs i EROFS.
- **Svakodnevne aplikacije:** pregledač, kancelarija, pošta, fotografije i muzika preko zrelih otvorenih projekata.

## Ukupan napredak prema specifikaciji

Specifikacija procenjuje NeoOS 1.0 na 5 350 čovek-meseci. Do sada urađeno kodiranje iznosi oko 1 380 inženjerskih sati, otprilike 8,6 čovek-meseci ili 0,16% te procene. To je samo seme jezgra. Deo toga može biti zamenjen prelaskom na Genode na seL4, pa je ovo gornja granica.
