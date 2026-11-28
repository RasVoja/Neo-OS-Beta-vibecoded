# NBPOS lista mogućnosti

[English](FEATURES.md) · **Srpski** · Ažurirano 28. novembra 2026.

Jedna strana sa svim što NBPOS (Neo Boing Platinum OS) radi ili planira, sa oznakom obima. Detalji su u [STATUS.sr.md](STATUS.sr.md) (etape), specifikaciji (PDF) i stranama plana na kraju.

## Pravilo obima

**Prvo Lite.** Neo Boing Platinum OS Lite na x86_64 i ARM-u preuzima sve iz ranije pune i pro verzije što softver može na današnjem hardveru. Cilj je da se sistem prvo dokaže na postojećoj platformi.

**Platforme: samo x86_64 i ARM.** Merila su Raspberry Pi 5 (minimum), PC sa AMD Zen 2 procesorom (referenca) i PC sa Intel Core 10. generacije; šta se od svakog očekuje je u odeljku 4.

Namenski Neo hardver koji ne postoji je odvojeni koncept (konceptni hardver). Nalazi se samo u poslednjem odeljku ove strane, a u dokumentima, video snimcima i primerima nosi vidljivu oznaku „koncept”, da se nikad ne pomeša sa onim što radi.

Oznake (u engleskoj verziji [done], [next], [lite], [concept]): **[gotovo]** radi i proverava se u CI-ju · **[u izradi]** gradi se sada · **[lite]** planirano za Lite, može odmah u plan · **[koncept]** traži hardver koji ne postoji (konceptni hardver).

Pravilo za oznake: sve što ne traži Neo hardver je **[lite]** i može odmah da uđe u plan razvoja.

## Na prvi pogled

| Oblast | [gotovo] | [u izradi] | [lite] |
| --- | --- | --- | --- |
| Jezgro i programi | Tvrđava na x86_64 i AArch64, korisnički programi, portovi, ovlašćenja | Genode na seL4 | Obnova, Hronos, Brava |
| Pokretanje i drajveri | ISO za BIOS i UEFI, sto na GOP ekranu, USB tastatura i miš, PCI, ACPI | Prvo paljenje na pravim PC-jima | Mreža, zvuk, diskovi, Raspberry Pi 5 (glavni ARM cilj) i Pi 4 |
| Radna površina | Prvi prozor, pločice Sat, Merač i Kalkulator, Dijana, Neo DOS 0.3 | Kompozitor, Pločice, Polica i Traka | Neo Kit, Neo Srebro, Prvi Sto, promena jezika u radu, Otvoreni prevod |
| Programi | | | Ormar Pro, Album (gThumb), FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), kancelarija, pošta, veb |
| Igre | | | Naše Neo igre, Flatpak igre, Arkada (MAME, DOSBox, ScummVM, RetroArch) |
| Muzika | | | HippoPlayer, qBittorrent, Hydrogen, treker, beatmaker |
| Pomoć | Beta priručnik, HTML i PDF vodič | | Program Pomoć, F1, slike pravog ekrana |

## 1. Radi danas [gotovo]

- Jezgro Fortress se podiže na x86_64 i AArch64 (QEMU), sa serijskom konzolom i probom paljenja u CI-ju.
- Izuzeci procesora, sopstvene tabele stranica, zaštitna stranica steka, upravljanje celom fizičkom memorijom, hip jezgra, tajmer, raspoređivač sa prekidanjem.
- Programi u korisničkom režimu, svaki u svom adresnom prostoru; pad zaustavlja samo taj program.
- Imenovani portovi za poruke i sposobnosti (capabilities); prstenovi u deljenoj memoriji za velike poruke.
- Rust biblioteka za programe (`neo-rt`), ELF učitavač, programi se učitavaju sa slike diska.
- Servisi u korisničkom režimu: konzola, tastatura (PS/2, virtio-input), ekran (framebuffer), virtio blok uređaj sa fajl sistemom samo za čitanje.
- Rasporedi tastature US i srpska latinica (č, ć, ž, š, đ), prebacivanje sa Alt+Shift.
- Neo DOS 0.3 sa volumenima BOOT: i DH0:, uređivanje reda i istorija, BOOTLOG, USB i PCI.
- Prvi prozor koji se vuče mišem, pločice Sat, Merač i Kalkulator, crtane preko zadnjeg bafera (bez treperenja); Dijana na Traci, sa savetima posle greške i laku noć pri gašenju; Neo meni (O sistemu, Pomoć, Laku noć); prozori se zatvaraju dugmetom za zatvaranje, umanjuju se na Traku i navedeni su na njoj.
- Pokretački hibridni ISO za BIOS i UEFI (GRUB, Multiboot2), proba u CI-ju sa SeaBIOS i OVMF.
- Radna površina na ekranu koji podesi pokretač (UEFI GOP ili VBE).
- USB tastatura i miš (xHCI, habovi, HID boot izveštaji, utikanje u radu na portovima i iza habova); sve PCI magistrale i visoki BAR-ovi; ACPI tabele (procesori, rutiranje prekida, gašenje).
- Proba Genode na seL4: neizmenjen Neo program radi kao Genode komponenta; server `neo_ports`.
- Vodič za saradnike, dvanaest početnih zadataka, GPLv3 ili novija.

## 2. U izradi [u izradi]

- Prvo paljenje na pravim PC-jevima (vidi [BUILD-ISO.sr.md](BUILD-ISO.sr.md)).
- Genode na seL4: buđenje signalom, deljena memorija, ručke u porukama, zatim Neo DOS.
- Radna površina: ostatak kompozitora, još pločica, menjanje veličine prozora, Polica i Traka, prevlačenje, Neo meni, komandna paleta, pa izgled Neo Srebro.

## 3. Planirano za Lite [lite]

**Sistem**
- Obnova (jedno potpisano A/B ažuriranje sa povratkom jednim klikom), Hronos (snimci, „vrati na juče”), Brava (privatnost po programu, dnevnik odlaznog saobraćaja), Čuvar (rezervne kopije), Traži (jedan lokalni indeks), Riznica (katalog programa otvorenog koda sa ponovljivom izgradnjom).
- Linux box (domen Aplikacije) za Flatpak i POSIX programe; Most (FEX/Box64, Wine, Proton) na ARM-u; Android soba (1.1).
- Nativni mrežni, zvučni i drajveri za skladište za uobičajen hardver; Linux kod drajvera samo u izolovanim komponentama.
- Neo Oživi: sistem sa USB ključa na starim 64-bitnim UEFI računarima i pločama klase Raspberry Pi 4/5, uz spisak proverenih računara.
- Komšiluk: slanje fajlova obližnjim računarima, pošta po komšiluku, pozajmica snage od većih računara sa NBPOS-om (Zadruga).
- RAD: u osnovnoj verziji, po nivoima ([RAD.sr.md](RAD.sr.md)).

**Sto i ponašanje**
- Neo Kit (jedan toolkit, prekidač Lako/Pro), Lux (port za skripte u svakom programu), Tipovi podataka (preko 1.000 formata, dekoderi u pesku), Poništi svuda.
- Samo stil Retro i nivo Mirno, izgled Neo Srebro, tema Prvi Sto, srpski (latinica i ćirilica) i engleski (UK i US), glas i „Pitaj Neo” sa lokalnim modelima.
- Zadaci, Blokovi (šabloni), nema dugmeta Sačuvaj, Skloni u stranu, taster Skok, univerzalni tasteri Premesti/Kopiraj/Svojstva/Ponovi/Poništi.
- **Jezik može da se menja dok sistem radi**, bez restarta i bez gubitka otvorenih dokumenata: iz stavke u traci menija i iz Podešavanja. Latinica i ćirilica se prebacuju odvojeno od jezika; raspored tastature je takođe odvojen. Programi prikazuju tekst samo iz jezičkih kataloga, nikad iz ugrađenih nizova. Programi iz Linux boxa prate promenu gde mogu, a mogu da traže restart.
- **Otvoreni prevod [lite]:** desni klik na dokument, veb stranu ili označen tekst (u Ormaru, Peru, Vidiku, Listaj i Golubu), pa *Prevedi na…* i jedan od velikih svetskih jezika: engleski, španski, francuski, nemački, portugalski, italijanski, ruski, kineski, japanski, korejski, arapski, hindi, turski i drugi, a srpski gde postoji model. Prevod se otvara kao novi dokument pored originala i čuva raspored gde format to dozvoljava. Mašina je **LibreTranslate** (AGPL-3.0, na sopstvenom računaru) sa modelima **Argos Translate** (MIT), kao lokalni servis, prvo u Linux boxu. Jezički paketi se preuzmu jednom i zatim rade bez mreže. Ništa ne ide u Google Translate ni u bilo koji drugi servis u oblaku; Brava pokazuje da servis nema pristup mreži kad su paketi na mestu. LibreTranslate se isporučuje kao zaseban program zbog svoje licence; licence se ponovo proveravaju pre spajanja koda.
- Dijana kao stanar i vodič koji ne smeta ([MASCOT.sr.md](MASCOT.sr.md)); Zmija kao drugi stanar za test build ([RESIDENTS.sr.md](RESIDENTS.sr.md)).

**Aplikacije (prvo P0)**
- Ormar (fajlovi, dva panela), Školjka, Neo BASIC, Pero, Abakus, Golub (pošta), Vidik (veb), Album, Gramofon, Bioskop, Rokovnik, Ključar, Arkada (emulatori i boxovi, sa emulatorom Classic Mac-a: [SYSTEM7-REFERENCE.sr.md](SYSTEM7-REFERENCE.sr.md)), Čuvar, Listaj.
- Dodato u novembru 2026: HippoPlayer kao audio engine, qBittorrent, Hydrogen i beatmaker, razmena sa trekerom ([MUSIC.sr.md](MUSIC.sr.md)); FilmCraft (video editor, zasnovan na Kdenlive-u), PhotoCraft (editor slika, zasnovan na GIMP-u i Inkscape-u) i Album (pregledač slika, zasnovan na gThumb-u) ([CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md)).
- Gotove slobodne igre preko Flatpaka u Linux boxu: SuperTuxKart, 0 A.D., Luanti (Minetest), Frozen Bubble, Battle for Wesnoth; Arkada sa MAME-om, DOSBox-om, ScummVM-om i RetroArch-om ([GAMES.sr.md](GAMES.sr.md)).
- Svaki program pokazuje jednostavno ime i kaže na čemu je zasnovan, na splash ekranu, u „O programu”, u pomoći i u dokumentaciji ([APP-NAMES.sr.md](APP-NAMES.sr.md)).
- **Neo Boing Platinum OS Lite za decu:** isti Lite sa profilom Dečja soba kao podrazumevanim, manjim skupom programa i zaključanim funkcijama za odrasle ([KIDS.sr.md](KIDS.sr.md)).

**Liste uzora**
- AmigaOS, klasični Mac, BeOS i Haiku, RISC OS, te navike modernog macOS-a i Unix sveta: vidi strane plana ispod.

## 3b. Prebačeno iz Pro u Lite (27. novembar 2026)

Sve što ne čeka potpuni hardver sada je Lite. Gde je Pro tekst tražio namenski hardver, Lite koristi softversku verziju.

| Pro ideja | U Lite-u |
| --- | --- |
| Sve igre ([GAMES.sr.md](GAMES.sr.md)) | Softverske verzije; Warblade Neo čeka licencu |
| Arkada | Emulatori: MAME, DOSBox, ScummVM, RetroArch, FS-UAE, Classic Mac; plus igre otvorenog koda preko Flatpaka |
| Paket Radionica (video, foto, crtanje) | FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), Album (gThumb) |
| Studijske zvučne mogućnosti | HippoPlayer, qBittorrent, Hydrogen, treker, beatmaker ([MUSIC.sr.md](MUSIC.sr.md)) |
| Neo škola, Neo Karte, Neo BASIC, kurs jezika, vesti, veza sa telefonom | Isti programi, na običnom hardveru (demoi već u `docs/html/sr/demo`) |
| Claude pomoć | **Daleka budućnost, nije garantovano.** Nije u planu za Lite; demo samo pokazuje ideju |
| Ormar Pro i Magelan (pravila i automatizacija), Zgusni (arhive), Pločice i Traka | Isto; prvi su posao na radnoj površini |
| Stilovi: Retro, Šok budućnosti i tema Kristal | Prvo Retro; Šok budućnosti i Kristal koliko radna površina dozvoli. Domaćin Kristala je Zmija ([RESIDENTS.sr.md](RESIDENTS.sr.md)) |
| Prekidač Lako/Pro, Dečja soba, veliki profil, pristupačnost | Lite, uz dečju verziju ([KIDS.sr.md](KIDS.sr.md)) |
| Zvučne teme (Neo osnovna, Kristal, Drvo, Čip) | Lite, softverski mikser |
| Pro alati: Hronos, Obnova, Brava, Čuvar, Traži, Riznica | Lite (odeljak 3 gore) |
| Grafika, AI i fizika koje je Pro tekst davao namenskim čipovima | Grafička kartica samog računara (Mesa, Vulkan) ili softversko iscrtavanje, lokalni modeli na procesoru, softverska fizika, emulatori |
| Pomoć i korisničko uputstvo | U sistemu, HTML i PDF ([HELP.sr.md](HELP.sr.md)) |

## 4. Performanse na današnjem hardveru (procena)

NBPOS Lite pravi se za x86_64 i ARM (AArch64) računare koji postoje danas. Tri mašine su merilo: **Raspberry Pi 5** kao minimum, PC sa **AMD Zen 2** procesorom kao referenca i PC sa **Intel Core 10. generacije** kao njen Intel ekvivalent.

Ovo su inženjerske procene za planiranu radnu površinu Lite 1.0, izvedene iz onoga što slični sistemi (Haiku, laka Linux okruženja, Raspberry Pi OS) postižu na istom hardveru. To su ciljevi, ne merenja: današnji kod pokazuje tek prvi sto, a prenos na Raspberry Pi 5 još nije počeo (ARM radi u QEMU `virt`). Izveštaji sa pravih računara ([REAL-PC-TEST.sr.md](REAL-PC-TEST.sr.md)) zameniće ih izmerenim brojevima.

| | **Raspberry Pi 5**, 8 GB (minimum) | **Intel Core 10. gen.** (Comet Lake, npr. Core i5-10400 sa grafikom UHD 630), 16 GB | **AMD Zen 2** (Ryzen 3000, npr. Ryzen 5 3600 sa grafičkom karticom srednje klase), 16 GB (referenca) |
| --- | --- | --- | --- |
| Procesor | 4 × Cortex-A76, 2,4 GHz | 6 jezgara, 12 niti, do 4,3 GHz | 6 jezgara, 12 niti, do 4,2 GHz |
| Od paljenja do stola | oko 10–15 s sa NVMe ili USB SSD-a, 20–30 s sa microSD kartice | oko 5–8 s sa SSD-a | oko 5–8 s sa SSD-a |
| Glatka radna površina na 60 Hz | 1080p sa svim efektima; 1440p i 4K sa smanjenim staklom i senkama | 1440p sa svim efektima; 4K sa manje efekata | 4K sa svim efektima; 1440p na 120–144 Hz |
| Pokazivač i prevlačenje prozora | 60 slika u sekundi, pokazivač kasni najviše jednu sliku | isto | isto, do 144 Hz na brzim monitorima |
| Otvorenih prozora pre nego što išta uspori | oko 15–20 | oko 40 | 50 i više |
| Pokretanje izvornog Neo programa (Ormar, Pero, Abakus) | manje od 1 s | manje od 0,3 s | manje od 0,3 s |
| Memorija za sto i nekoliko programa | oko 400–600 MB | oko 400–600 MB | oko 400–600 MB |
| Programi iz Linux boxa (Flatpak) | kancelarija, pregledač sa desetak kartica, Album (gThumb), PhotoCraft (GIMP) bez problema; FilmCraft (Kdenlive) 1080p uz proxy fajlove; 3–4 teška programa istovremeno | svi; FilmCraft 4K uz proxy fajlove | svi; FilmCraft 4K, renderovanje na grafičkoj kartici |
| Igre | Frozen Bubble, Wesnoth i Neo 2D igre punom brzinom; SuperTuxKart i Luanti 720p–1080p na niskim podešavanjima, 30–60 fps; 0 A.D. male mape | SuperTuxKart i Luanti 1080p na srednjim; 0 A.D. srednje mape | 1080p–1440p na visokim |
| Arkada (emulatori) | DOSBox, ScummVM, FS-UAE (Amiga 500 i 1200) punom brzinom; MAME za većinu igara iz 1980-ih i ranih 1990-ih; RetroArch do PlayStation 1 | MAME za skoro sve 2D igre; RetroArch do Nintendo 64 i Dreamcast | do PlayStation 2 i GameCube u većini igara |
| Lokalni modeli („Pitaj Neo”, govor) | govor sa malim modelima; jezički model od 1–3 B, nekoliko tokena u sekundi | model od 3–4 B, oko 8–12 tokena u sekundi | model od 7–8 B, oko 6–10 tokena u sekundi na procesoru, brže na grafičkoj kartici |
| Otvoreni prevod, jedna strana (~500 reči) | oko 5–15 s | oko 1–3 s | oko 1–3 s |
| Potrošnja na stolu | oko 4–8 W | oko 25–40 W (ceo PC) | oko 40–60 W sa grafičkom karticom |

Napomene:
- **Raspberry Pi 5 sa 4 GB** pokreće isti sto, sa manje istovremeno otvorenih programa iz Linux boxa. Raspberry Pi 4 i stari PC računari (Neo Oživi, otprilike od 2012) pokreću Lite sa isključenim efektima.
- **Zen 2 sa ugrađenom grafikom** (Ryzen 4000 i 5000 APU, npr. Ryzen 5 4600G, i laptopovi sa Ryzen 5 4500U) su između Intel i Ryzen kolone: procesor reference, grafika blizu Intel UHD 630 ili malo bolja.
- Intel kolona je ekvivalent AMD reference po procesoru; razlika između te dve kolone je uglavnom grafička kartica.

## Strane plana

[AMIGA.sr.md](AMIGA.sr.md) · [CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md) · [BEOS-HAIKU.sr.md](BEOS-HAIKU.sr.md) · [RISC-OS.sr.md](RISC-OS.sr.md) · [MUSIC.sr.md](MUSIC.sr.md) · [CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md) · [APP-NAMES.sr.md](APP-NAMES.sr.md) · [KIDS.sr.md](KIDS.sr.md) · [RAD.sr.md](RAD.sr.md) · [GAMES.sr.md](GAMES.sr.md) · [HELP.sr.md](HELP.sr.md) · [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md) · [RESIDENTS.sr.md](RESIDENTS.sr.md) · [SYSTEM7-REFERENCE.sr.md](SYSTEM7-REFERENCE.sr.md) · [BUILD-ISO.sr.md](BUILD-ISO.sr.md) · [HANDOFF.sr.md](HANDOFF.sr.md)

## Konceptni hardver (odvojeno, nije potreban)

> **Konceptni hardver.** Vizija budućeg namenskog računara. Ništa od toga ne postoji, ništa gore ne zavisi od toga, i sve mogućnosti gore rade na običnim x86_64 i ARM računarima. Ni Claude integracija nije deo toga: to je ideja za daleku budućnost, nije garantovano.

- Ploče **Neo 500, 2000 i 4000**, ručni računar **NeoPocket** i Neo 100, računar u tastaturi, sa izmenljivom Neo karticom i Autokonfigom.
- Neo paket čipova (grafika VSA-6000, Zrak, Panorama, Njutn fizika, Kopernik, Neuron AI, Orfej zvuk, Kurir, Hronos i Sef), FPGA jezgra (servis Kameleon, jezgra u Arkadi) i x86 modul Satelit.
- Hardverski prekidači privatnosti sa LED-om, kamere Oko, kertridži i Džepić, ležište za UHD Blu-ray i M-DISC, Sunce stanica, Neo Automat, e-ink Dvolik.
- Hardverski dodaci za mogućnosti sistema: RAD: koji preživljava nestanak struje, efekti stakla i fizike u hardveru. Na običnim računarima iste mogućnosti rade u softveru (odeljak 3b).

Detalji: [CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md).
