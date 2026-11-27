# Neo Boing Platinum OS

[English](README.md) · **Srpski**

Neo Boing Platinum OS (NBPOS) je operativni sistem sa ljudskim licem: jedan sistem, isti za dete, odraslog i profesionalca. Treba da bude jedini računar u kući: pošta, veb, kancelarija, fotografije, muzika, učenje, igre i retro, bez naloga i bez telemetrije.

**Sistem stoji sam za sebe.** Njegovo prvo izdanje, **Neo Boing Platinum OS Lite**, pravi se za obične računare koji postoje danas: opšte **x86_64 PC računare** i **ARM ploče**. Nikakav poseban hardver nije potreban. Odvojeni *konceptni računar* (Neo 500, 2000, 4000) opisan je na kraju ove strane kao opciona ideja za budućnost; nije potreban da bi se sistem gradio, pokretao ili koristio.

> **Stanje: kod je ovde i otvoren za doprinose.** Jezgro *Tvrđava* (Fortress) podiže se na x86_64 i AArch64 u QEMU-u i sa pokretačkog ISO-a (BIOS i UEFI). Pokreće programe pisane u Rustu u korisničkom režimu, sa ovlašćenjima (capabilities) i portovima za poruke, i ima tastaturu (US i srpska latinica), miša (PS/2 i USB), ekran sa prozorima, disk, terminal i Neo DOS komandnu liniju. Prvi Neo program radi i na Genode-u sa seL4 (vidi [genode/](genode/README.sr.md)). U [docs/STATUS.sr.md](docs/STATUS.sr.md) se prati šta je urađeno i planirano; [docs/FEATURES.sr.md](docs/FEATURES.sr.md) navodi sve mogućnosti po obimu; [docs/PROGRESS.md](docs/PROGRESS.md) je dnevnik razvoja korak po korak (na engleskom).

> **O imenu.** Do 18. novembra 2026. sistem se zvao *NeoOS* (ili *Neo OS*). To ime koriste i drugi, nepovezani projekti, pa se sada zove **Neo Boing Platinum OS**. *Boing* je ovde pozdrav duhu Amige; projekat ne koristi logo Amiga Boing Ball, koji je zaštitni znak svog vlasnika. Neka imena programa u kodu i stariji video snimci još pokazuju staro ime.

## Korisničko uputstvo i pomoć

| | Srpski | English |
| --- | --- | --- |
| **Korisnički vodič (počni ovde)**: sve strane planova i mogućnosti kao veb stranice | [guide/html-sr/index.html](guide/html-sr/index.html) | [guide/html-en/index.html](guide/html-en/index.html) |
| Isti vodič kao jedan PDF | [NBPOS-vodic-sr.pdf](guide/NBPOS-vodic-sr.pdf) | [NBPOS-guide-en.pdf](guide/NBPOS-guide-en.pdf) |
| Beta priručnik radne površine (u izradi) | [neo-prirucnik-beta.html](docs/html/sr/neo-prirucnik-beta.html) | [neo-manual-beta.html](docs/html/en/neo-manual-beta.html) |
| Kako će pomoć raditi u sistemu (F1, slike pravog ekrana, bez mreže) | [docs/HELP.sr.md](docs/HELP.sr.md) | [docs/HELP.md](docs/HELP.md) |
| Pravljenje pokretačke slike i prvo paljenje na PC-ju | [docs/BUILD-ISO.sr.md](docs/BUILD-ISO.sr.md) | [docs/BUILD-ISO.md](docs/BUILD-ISO.md) |

GitHub prikazuje HTML kao izvorni kod. Preuzmi repozitorijum kao ZIP (ili samo fasciklu `guide/`) i otvori `index.html` u pregledaču; ne trebaju ni server ni mreža. `python3 scripts/build-guide.py` ponovo gradi vodič (HTML i PDF) iz Markdown fajlova u `docs/`.

## Neo Boing Platinum OS Lite: na čemu radi

Lite je deo koji se gradi i testira sada. Preuzima sve iz ranije pune i Pro verzije što softver može na današnjem hardveru, uključujući igre, kreativne i muzičke programe, Ormar Pro, Zgusni, Pločice i sistem pomoći.

- **Opšti x86_64 PC**: prvi razvojni cilj, u QEMU-u i sa USB-a ili CD-a (hibridni ISO, BIOS i UEFI).
- **Opšti ARM (AArch64)**: sada QEMU `virt`, zatim ploče klase Raspberry Pi 4/5.
- **Neo Revive**: sistem sa USB-a na starim 64-bitnim UEFI računarima (otprilike od 2012, 2 GB RAM-a ili više).
- **Telefoni**: eksperimentalno, samo gde postoje otvoreni drajveri.

Programi koji već postoje za Linux (igre, Kdenlive, GIMP, gThumb, qBittorrent, Hydrogen i drugi) rade u **Linux kutiji** preko Flatpaka, pod Neo imenima i sa Neo pravilima privatnosti, dok ne nastanu izvorne verzije. Vidi [docs/FEATURES.sr.md](docs/FEATURES.sr.md), [docs/GAMES.sr.md](docs/GAMES.sr.md), [docs/CREATIVE-APPS.sr.md](docs/CREATIVE-APPS.sr.md) i [docs/MUSIC.sr.md](docs/MUSIC.sr.md).

## Demoi sistema

Simulacije u pregledaču koje pokazuju mogućnosti samog sistema. Prave se da pokažu kakav će sistem biti; nisu sam sistem. Preuzmi fasciklu i otvori fajl u pregledaču.

| | Srpski | English |
| --- | --- | --- |
| Svi demoi (počni ovde) | [neo-demoi.html](docs/html/sr/demo/neo-demoi.html) | [neo-demos.html](docs/html/en/demo/neo-demos.html) |
| Neo sto | [neo-demo.html](docs/html/sr/demo/neo-demo.html) | [neo-demo.html](docs/html/en/demo/neo-demo.html) |
| Ormar Pro i Dvojac (menadžer fajlova) | [neo-ormar.html](docs/html/sr/demo/neo-ormar.html) | [neo-cabinet.html](docs/html/en/demo/neo-cabinet.html) |
| Ormar Pro: Magelan (pravila i automatizacija) | [neo-ormar-magellan.html](docs/html/sr/demo/neo-ormar-magellan.html) | [neo-cabinet-magellan.html](docs/html/en/demo/neo-cabinet-magellan.html) |
| Neo DOS (komandna linija) | [neo-dos.html](docs/html/sr/demo/neo-dos.html) | [neo-dos.html](docs/html/en/demo/neo-dos.html) |
| Pločice i Traka | [neo-ploce.html](docs/html/sr/demo/neo-ploce.html) | [neo-tiles.html](docs/html/en/demo/neo-tiles.html) |
| Zgusni (kompresija i arhive) | [neo-zgusni.html](docs/html/sr/demo/neo-zgusni.html) | [neo-crunch.html](docs/html/en/demo/neo-crunch.html) |
| Prvi Sto (retro tema) | [neo-prvi-sto.html](docs/html/sr/demo/neo-prvi-sto.html) | [neo-first-desk.html](docs/html/en/demo/neo-first-desk.html) |
| Škola, jezici, vesti, Claude pomoć, veza sa telefonom | [docs/html/sr/demo](docs/html/sr/demo) | [docs/html/en/demo](docs/html/en/demo) |
| Dijana, stanar (pixel i vektorski crteži) | [assets/dijana/](assets/dijana/) | [assets/dijana/](assets/dijana/) |

## Dokumenti o sistemu

| Dokument | Srpski | English |
| --- | --- | --- |
| Neo Boing Platinum OS 1.0: specifikacija i plan razvoja (njena poglavlja o hardveru su konceptni hardver) | [PDF](docs/pdf/sr/Neo-Boing-Platinum-OS-1.0-specifikacija.pdf) · [veb](docs/html/sr/neoos-1-0.html) | [PDF](docs/pdf/en/Neo-Boing-Platinum-OS-1.0-specification.pdf) · [veb](docs/html/en/neoos-1-0-specification.html) |
| Šta Neo još ume | [PDF](docs/pdf/sr/Neo-sta-jos-ume.pdf) | [PDF](docs/pdf/en/Neo-what-else-it-can-do.pdf) |
| Dosadilo vam je? (šta sistem uklanja iz svakodnevnog rada) | [PDF](docs/pdf/sr/Neo-dosadilo-vam-je.pdf) | [PDF](docs/pdf/en/Neo-fed-up.pdf) |

Specifikacija je radni nacrt. Sve brojke u njoj su inženjerske procene, a ne merenja.

## Arhitektura u jednoj tabeli

| Sloj | Šta je |
| --- | --- |
| Aplikacije i igre | Menadžer fajlova, pregledač, kancelarija, Arkada, programi iz kataloga Riznica, Linux programi u Linux kutiji |
| Neo Kit i Lux | Jedan alat za prozore, teme, Datatipovi, port za skripte u svakoj aplikaciji |
| Sistemski servisi | Exec-N (portovi za poruke), Dispečer, Hronos (snimci stanja), Obnova (A/B ažuriranja), Brava (privatnost), Čuvar (rezervne kopije), Nađi |
| Drajveri | Izvorni drajveri u Rustu; kod Linux drajvera samo u izolovanim komponentama |
| Tvrđava | Mikrojezgro klase seL4 koje deli memoriju, procesorsko vreme i ovlašćenja |
| Pokretanje i Sef | Pokretač (danas GRUB ili UEFI), potpisana A/B slika sistema, ključevi |

Pravila dizajna: odziv pre propusnosti; jedan sistem, jedan API, jedan izgled; svaka aplikacija se može skriptovati; malo i razumljivo, sa budžetom veličine i memorije za svaku sistemsku aplikaciju. Sistem se piše u Rustu.

Izgled radne površine, *Neo Srebro*, prati klasični Mac OS uz najbolje iz BeOS-a, Haiku-a, MorphOS-a i Amige. Svako ime, ikona i crtež su delo projekta. Srpski (latinica i ćirilica) i engleski (UK i US) su prvi jezici, a komplet za lokalizaciju olakšava dodavanje bilo kog drugog jezika.

## Planovi i liste mogućnosti

| Strana | Šta pokriva |
| --- | --- |
| [docs/FEATURES.sr.md](docs/FEATURES.sr.md) | Sve što sistem radi ili planira, sa oznakama [gotovo], [u izradi], [lite] ili [koncept] |
| [docs/STATUS.sr.md](docs/STATUS.sr.md) | Etape i napredak |
| [docs/GAMES.sr.md](docs/GAMES.sr.md) | Igre u Lite-u: naše igre, Arkada (DOSBox, ScummVM, RetroArch) i igre otvorenog koda preko Flatpaka |
| [docs/CREATIVE-APPS.sr.md](docs/CREATIVE-APPS.sr.md) | FilmCraft (Kdenlive), PhotoCraft (GIMP i Inkscape), Album (gThumb) |
| [docs/MUSIC.sr.md](docs/MUSIC.sr.md) | HippoPlayer, qBittorrent, Hydrogen, beatmaker, razmena sa trekerom |
| [docs/APP-NAMES.sr.md](docs/APP-NAMES.sr.md) | Jednostavna imena programa i na čemu je koji zasnovan |
| [docs/KIDS.sr.md](docs/KIDS.sr.md) | Neo Boing Platinum OS Lite za decu |
| [docs/RAD.sr.md](docs/RAD.sr.md) | RAD: u osnovnoj verziji |
| [docs/AMIGA.sr.md](docs/AMIGA.sr.md), [docs/CLASSIC-MAC.sr.md](docs/CLASSIC-MAC.sr.md), [docs/BEOS-HAIKU.sr.md](docs/BEOS-HAIKU.sr.md), [docs/RISC-OS.sr.md](docs/RISC-OS.sr.md), [docs/SYSTEM7-REFERENCE.sr.md](docs/SYSTEM7-REFERENCE.sr.md) | Ideje i programi preuzeti iz drugih sistema |
| [docs/RESIDENTS.sr.md](docs/RESIDENTS.sr.md), [docs/MASCOT.sr.md](docs/MASCOT.sr.md) | Stanari: Dijana, i Zmija kao drugi |
| [docs/HANDOFF.sr.md](docs/HANDOFF.sr.md), [docs/SCHEDULED-CODING.sr.md](docs/SCHEDULED-CODING.sr.md) | Kako nastaviti kodiranje, i kodiranje po rasporedu na sopstvenom PC-ju |

## Na temeljima otvorenog koda

Sve što već postoji kao zreo otvoren projekat uzima se kao temelj, ispočetka se piše samo ono što je novo, a izmene se vraćaju izvornim projektima. Kandidati su Genode na seL4, kod Linux drajvera u izolovanim komponentama, Btrfs i EROFS, Mesa, Smithay, Haiku Interface Kit, PipeWire, Flatpak, QEMU, LLVM i Rust.

## Kako pomoći

Počnite od [CONTRIBUTING.sr.md](CONTRIBUTING.sr.md): šta vam treba, kako se gradi i pokreću testovi podizanja i koja pravila izmena mora da poštuje. [docs/STARTER-TASKS.sr.md](docs/STARTER-TASKS.sr.md) navodi dvanaest početnih zadataka, svaki sa svojim testom, i svaki je i issue. Pitanja, ideje i komentari na specifikaciju takođe su dobrodošli kao issues.

---

## Odvojeno: koncept idealnog računara (konceptni hardver)

> **Konceptni hardver. Nije dostupan i nije potreban.** Sve u ovom odeljku je vizija budućeg računara. Ništa od toga ne postoji, i sistem opisan gore ne zavisi od toga.

**Neo 500, 2000 i 4000** (konceptni hardver) su ideja idealnog kućnog računara koji bi dolazio sa Neo Boing Platinum OS-om: brz ARM procesor sa velikim FPGA, Neo paket čipova (grafika VSA-6000, Neuron, fizika Njutn i drugi), sistem u ROM-u, NVMe kertridži, opcioni x86 modul Satelit i Neo 100, računar u tastaturi. Na takvoj mašini unapred instaliran sistem bio bi **jedna mogućnost od mnogih**, a ne uslov; isti sistem radi na bilo kom PC-ju ili ARM ploči. Pravilo koje ih drži odvojeno je u [docs/CONCEPT-COMPUTER.sr.md](docs/CONCEPT-COMPUTER.sr.md).

| Konceptni dokument (konceptni hardver) | Srpski | English |
| --- | --- | --- |
| Porodica računara Neo (pregled) | [PDF](docs/pdf/sr/Neo-pregled.pdf) · [veb](docs/html/sr/neo-porodica.html) | [PDF](docs/pdf/en/Neo-overview.pdf) · [web](docs/html/en/neo-overview.html) |
| Promo brošura | [PDF](docs/pdf/sr/Neo-promo-brosura.pdf) · [veb](docs/html/sr/neo-brosura.html) | [PDF](docs/pdf/en/Neo-promo-brochure.pdf) · [web](docs/html/en/neo-promo-brochure.html) |
| Neo iznutra (tehnička knjiga; tema Kristal sa DJ Nick-om je koncept, mogućnost, a ne dogovor) | [PDF](docs/pdf/sr/Neo-tehnicka-knjiga.pdf) · [veb](docs/html/sr/neo-tehnicka-knjiga.html) | [PDF](docs/pdf/en/Neo-technical-book.pdf) · [web](docs/html/en/neo-technical-book.html) |
| Neo identitet | [veb](docs/html/sr/neo-identitet.html) | [web](docs/html/en/neo-identity.html) |
| Neo za svakog (veličine uređaja od ručnog do Neo 4000) | [demo](docs/html/sr/demo/neo-za-svakog.html) | [demo](docs/html/en/demo/neo-for-everyone.html) |

Ovi dokumenti su pisani pre nego što su sistem i koncept razdvojeni, pa mešaju oboje. Biće podeljeni fajl po fajl u kasnijoj izmeni; do tada je sve u njima što traži Neo hardver konceptni hardver.

## Licenca

Neo Boing Platinum OS je slobodan softver pod GNU opštom javnom licencom (GPL), verzija 3 ili (po tvom izboru) bilo koja kasnija. Vidi [LICENSE](LICENSE).
