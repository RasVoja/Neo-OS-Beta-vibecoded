# NBPOS lista mogućnosti

[English](FEATURES.md) · **Srpski** · Ažurirano 27. novembra 2026.

Jedna strana sa svim što NBPOS (Neo Boing Platinum OS) radi ili planira, sa oznakom obima. Detalji su u [STATUS.sr.md](STATUS.sr.md) (etape), specifikaciji (PDF) i stranama plana na kraju.

## Pravilo obima

**Prvo Lite.** Neo Boing Platinum OS Lite na x86_64 i ARM-u preuzima sve iz ranije pune i pro verzije što softver može na današnjem hardveru. Cilj je da se sistem prvo dokaže na postojećoj platformi.

Sve što traži Neo hardver koji ne postoji (Neo 500, 2000, 4000, Neo paket, FPGA jezgra: konceptni hardver) ostaje **concept art**. Ovde je u posebnom odeljku, a u dokumentima, video snimcima i primerima nosi vidljivu oznaku „koncept”, da se nikad ne pomeša sa onim što radi.

Oznake (u engleskoj verziji [done], [next], [lite], [concept]): **[gotovo]** radi i proverava se u CI-ju · **[u izradi]** gradi se sada · **[lite]** planirano za Lite, može odmah u plan · **[koncept]** traži hardver koji ne postoji (konceptni hardver).

Pravilo za oznake: sve što ne traži Neo hardver je **[lite]** i može odmah da uđe u plan razvoja.

## Na prvi pogled

| Oblast | [gotovo] | [u izradi] | [lite] | [koncept] |
| --- | --- | --- | --- | --- |
| Jezgro i programi | Tvrđava na x86_64 i AArch64, korisnički programi, portovi, ovlašćenja | Genode na seL4 | Obnova, Hronos, Brava | |
| Pokretanje i drajveri | ISO za BIOS i UEFI, sto na GOP ekranu, USB tastatura i miš, PCI, ACPI | Prvo paljenje na pravim PC-jima | Mreža, zvuk, diskovi, Raspberry Pi 4/5 | Neo ploče, modul Satelit |
| Radna površina | Prvi prozor, pločica Sat, Neo DOS 0.2 | Kompozitor, Pločice, Polica i Traka | Neo Kit, Neo Srebro, Prvi Sto, promena jezika u radu | Hardverski efekti stakla |
| Programi | | | Ormar Pro, Album (gThumb), FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), kancelarija, pošta, veb | |
| Igre | | | Naše Neo igre, Flatpak igre, Arkada (MAME, DOSBox, ScummVM, RetroArch) | FPGA jezgra |
| Muzika | | | HippoPlayer, qBittorrent, Hydrogen, treker, beatmaker | Zvučni čip |
| Pomoć | Beta priručnik, HTML i PDF vodič | | Program Pomoć, F1, slike pravog ekrana | |

## 1. Radi danas [gotovo]

- Jezgro Fortress se podiže na x86_64 i AArch64 (QEMU), sa serijskom konzolom i probom paljenja u CI-ju.
- Izuzeci procesora, sopstvene tabele stranica, zaštitna stranica steka, upravljanje celom fizičkom memorijom, hip jezgra, tajmer, raspoređivač sa prekidanjem.
- Programi u korisničkom režimu, svaki u svom adresnom prostoru; pad zaustavlja samo taj program.
- Imenovani portovi za poruke i sposobnosti (capabilities); prstenovi u deljenoj memoriji za velike poruke.
- Rust biblioteka za programe (`neo-rt`), ELF učitavač, programi se učitavaju sa slike diska.
- Servisi u korisničkom režimu: konzola, tastatura (PS/2, virtio-input), ekran (framebuffer), virtio blok uređaj sa fajl sistemom samo za čitanje.
- Rasporedi tastature US i srpska latinica (č, ć, ž, š, đ), prebacivanje sa Alt+Shift.
- Neo DOS 0.2 sa volumenima BOOT: i DH0:, uređivanje reda i istorija.
- Prvi prozor koji se vuče mišem i pločica Sat.
- Pokretački hibridni ISO za BIOS i UEFI (GRUB, Multiboot2), proba u CI-ju sa SeaBIOS i OVMF.
- Radna površina na ekranu koji podesi pokretač (UEFI GOP ili VBE).
- USB tastatura i miš (xHCI, habovi, HID boot izveštaji); sve PCI magistrale i visoki BAR-ovi; ACPI tabele (procesori, rutiranje prekida, gašenje).
- Proba Genode na seL4: neizmenjen Neo program radi kao Genode komponenta; server `neo_ports`.
- Vodič za saradnike, dvanaest početnih zadataka, GPLv3 ili novija.

## 2. U izradi [u izradi]

- Prvo paljenje na pravim PC-jevima (vidi [BUILD-ISO.sr.md](BUILD-ISO.sr.md)).
- Genode na seL4: buđenje signalom, deljena memorija, ručke u porukama, zatim Neo DOS.
- Radna površina: kompozitor, pločice (Sat, Kalkulator, Merač), Polica i Traka, prevlačenje, Neo meni, komandna paleta, pa izgled Neo Srebro.

## 3. Planirano za Lite [lite]

**Sistem**
- Obnova (jedno potpisano A/B ažuriranje sa povratkom jednim klikom), Hronos (snimci, „vrati na juče”), Brava (privatnost po programu, dnevnik odlaznog saobraćaja), Čuvar (rezervne kopije), Traži (jedan lokalni indeks), Riznica (katalog programa otvorenog koda sa ponovljivom izgradnjom).
- Linux box (domen Aplikacije) za Flatpak i POSIX programe; Most (FEX/Box64, Wine, Proton) na ARM-u; Android soba (1.1).
- Nativni mrežni, zvučni i drajveri za skladište za uobičajen hardver; Linux kod drajvera samo u izolovanim komponentama.
- Neo Oživi: sistem sa USB ključa na starim 64-bitnim UEFI računarima i pločama klase Raspberry Pi 4/5, uz spisak proverenih računara.
- Komšiluk: slanje fajlova obližnjim računarima, pošta po komšiluku, pozajmica snage od većih Neo računara (Zadruga).
- RAD: u osnovnoj verziji, po nivoima ([RAD.sr.md](RAD.sr.md)).

**Sto i ponašanje**
- Neo Kit (jedan toolkit, prekidač Lako/Pro), Lux (port za skripte u svakom programu), Tipovi podataka (preko 1.000 formata, dekoderi u pesku), Poništi svuda.
- Samo stil Retro i nivo Mirno, izgled Neo Srebro, tema Prvi Sto, srpski (latinica i ćirilica) i engleski (UK i US), glas i „Pitaj Neo” sa lokalnim modelima.
- Zadaci, Blokovi (šabloni), nema dugmeta Sačuvaj, Skloni u stranu, taster Skok, univerzalni tasteri Premesti/Kopiraj/Svojstva/Ponovi/Poništi.
- **Jezik može da se menja dok sistem radi**, bez restarta i bez gubitka otvorenih dokumenata: iz stavke u traci menija i iz Podešavanja. Latinica i ćirilica se prebacuju odvojeno od jezika; raspored tastature je takođe odvojen. Programi prikazuju tekst samo iz jezičkih kataloga, nikad iz ugrađenih nizova. Programi iz Linux boxa prate promenu gde mogu, a mogu da traže restart.
- Dijana kao stanar i vodič koji ne smeta ([MASCOT.sr.md](MASCOT.sr.md)); Zmija kao drugi stanar za test build ([RESIDENTS.sr.md](RESIDENTS.sr.md)).

**Aplikacije (prvo P0)**
- Ormar (fajlovi, dva panela), Školjka, Neo BASIC, Pero, Abakus, Golub (pošta), Vidik (veb), Album, Gramofon, Bioskop, Rokovnik, Ključar, Arkada (emulatori i boxovi umesto FPGA, sa emulatorom Classic Mac-a: [SYSTEM7-REFERENCE.sr.md](SYSTEM7-REFERENCE.sr.md)), Čuvar, Listaj.
- Dodato u novembru 2026: HippoPlayer kao audio engine, qBittorrent, Hydrogen i beatmaker, razmena sa trekerom ([MUSIC.sr.md](MUSIC.sr.md)); FilmCraft (video editor, zasnovan na Kdenlive-u), PhotoCraft (editor slika, zasnovan na GIMP-u i Inkscape-u) i Album (pregledač slika, zasnovan na gThumb-u) ([CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md)).
- Gotove slobodne igre preko Flatpaka u Linux boxu: SuperTuxKart, 0 A.D., Luanti (Minetest), Frozen Bubble, Battle for Wesnoth; Arkada sa MAME-om, DOSBox-om, ScummVM-om i RetroArch-om ([GAMES.sr.md](GAMES.sr.md)).
- Svaki program pokazuje jednostavno ime i kaže na čemu je zasnovan, na splash ekranu, u „O programu”, u pomoći i u dokumentaciji ([APP-NAMES.sr.md](APP-NAMES.sr.md)).
- **Neo Boing Platinum OS Lite za decu:** isti Lite sa profilom Dečja soba kao podrazumevanim, manjim skupom programa i zaključanim funkcijama za odrasle ([KIDS.sr.md](KIDS.sr.md)).

**Liste uzora**
- AmigaOS, klasični Mac, BeOS i Haiku, RISC OS, te navike modernog macOS-a i Unix sveta: vidi strane plana ispod.

## 3b. Prebačeno iz Pro u Lite (27. novembar 2026)

Sve što ne čeka potpuni hardver sada je Lite. Gde je Pro tekst imenovao Neo čip, Lite koristi softversku verziju.

| Pro ideja | U Lite-u |
| --- | --- |
| Sve igre ([GAMES.sr.md](GAMES.sr.md)) | Softverske verzije; Warblade Neo čeka licencu |
| Arkada sa FPGA jezgrima | Arkada sa emulatorima: MAME, DOSBox, ScummVM, RetroArch, FS-UAE, Classic Mac; plus igre otvorenog koda preko Flatpaka |
| Paket Radionica (video, foto, crtanje) | FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), Album (gThumb) |
| Studijske zvučne mogućnosti | HippoPlayer, qBittorrent, Hydrogen, treker, beatmaker ([MUSIC.sr.md](MUSIC.sr.md)) |
| Neo škola, Neo Karte, Neo BASIC, kurs jezika, vesti, veza sa telefonom, Claude pomoć | Isti programi, na običnom hardveru (demoi već u `docs/html/sr/demo`) |
| Ormar Pro i Magelan (pravila i automatizacija), Zgusni (arhive), Pločice i Traka | Isto; prvi su posao na radnoj površini |
| Stilovi: Retro, Šok budućnosti i tema Kristal | Prvo Retro; Šok budućnosti i Kristal koliko radna površina dozvoli. Domaćin Kristala je Zmija ([RESIDENTS.sr.md](RESIDENTS.sr.md)) |
| Prekidač Lako/Pro, Dečja soba, veliki profil, pristupačnost | Lite, uz dečju verziju ([KIDS.sr.md](KIDS.sr.md)) |
| Zvučne teme (Neo osnovna, Kristal, Drvo, Čip) | Lite, softverski mikser |
| Pro alati: Hronos, Obnova, Brava, Čuvar, Traži, Riznica | Lite (odeljak 3 gore) |
| Grafika VSA-6000, Neuron AI, Njutn fizika, FPGA Kameleon | Softversko iscrtavanje, lokalni modeli na procesoru, softverska fizika, emulatori. Sami čipovi: koncept ([CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md)) |
| Pomoć i korisničko uputstvo | U sistemu, HTML i PDF ([HELP.sr.md](HELP.sr.md)) |

## 4. Samo concept art [koncept]

- Ploče Neo 500 / 2000 / 4000 (konceptni hardver), Neo 100 računar u tastaturi, izmenljiva Neo kartica i Autokonfig, Neo paket (VSA-6000, Zrak, Panorama, Njutn, Kopernik, Neuron, Orfej, Kurir, Hronos i Sef čipovi).
- FPGA jezgra (servis Kameleon, jezgra u Arkadi): Lite koristi emulatore.
- Satelit x86 modul, hardverski prekidači privatnosti sa LED-om, kamere Oko, kertridži i Džepić, ležište za UHD Blu-ray i M-DISC, Sunce stanica, Neo Automat, e-ink Dvolik.
- RAD: koji preživljava nestanak struje (ultrakondenzator ili baterija) i hardverski potpomognuti efekti stakla i fizike.

## Strane plana

[AMIGA.sr.md](AMIGA.sr.md) · [CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md) · [BEOS-HAIKU.sr.md](BEOS-HAIKU.sr.md) · [RISC-OS.sr.md](RISC-OS.sr.md) · [MUSIC.sr.md](MUSIC.sr.md) · [CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md) · [APP-NAMES.sr.md](APP-NAMES.sr.md) · [KIDS.sr.md](KIDS.sr.md) · [RAD.sr.md](RAD.sr.md) · [GAMES.sr.md](GAMES.sr.md) · [HELP.sr.md](HELP.sr.md) · [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md) · [RESIDENTS.sr.md](RESIDENTS.sr.md) · [SYSTEM7-REFERENCE.sr.md](SYSTEM7-REFERENCE.sr.md) · [BUILD-ISO.sr.md](BUILD-ISO.sr.md) · [HANDOFF.sr.md](HANDOFF.sr.md)
