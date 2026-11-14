# NeoOS

[English](README.md) · **Srpski**

NeoOS je operativni sistem porodice računara Neo. Cilj je jedan sistem sa ljudskim licem, isti za dete, odraslog i profesionalca, i isti od ručnog uređaja do radne stanice. Treba da bude jedini računar u kući: pošta, veb, kancelarija, fotografije, muzika, učenje, igre i retro, bez naloga i bez telemetrije.

> **Stanje: rani rad, uskoro otvoreno kodiranje.** Prvo jezgro, *Tvrđava* (Fortress), podiže se na x86_64 u QEMU-u. Pokreće programe pisane u Rustu u korisničkom režimu, sa ovlašćenjima (capabilities), portovima za poruke i servisom konzole. Kod se za sada razvija u privatnom repozitorijumu i seli se ovde kada dostigne prag za doprinose opisan u [docs/STATUS.sr.md](docs/STATUS.sr.md). Na toj strani se prati i šta je urađeno, a šta planirano po oblastima.

## Dokumenti o cilju

| Dokument | Srpski | English |
| --- | --- | --- |
| NeoOS 1.0: specifikacija i plan razvoja | [PDF](docs/pdf/sr/NeoOS-1.0-specifikacija.pdf) | [PDF](docs/pdf/en/NeoOS-1.0-specification.pdf) |
| Porodica računara Neo (pregled) | [PDF](docs/pdf/sr/Neo-pregled.pdf) | [PDF](docs/pdf/en/Neo-overview.pdf) |
| Promo brošura | [PDF](docs/pdf/sr/Neo-promo-brosura.pdf) | [PDF](docs/pdf/en/Neo-promo-brochure.pdf) |
| Šta Neo još ume | [PDF](docs/pdf/sr/Neo-sta-jos-ume.pdf) | [PDF](docs/pdf/en/Neo-what-else-it-can-do.pdf) |
| Dosadilo vam je? (šta Neo uklanja iz svakodnevnog rada) | [PDF](docs/pdf/sr/Neo-dosadilo-vam-je.pdf) | [PDF](docs/pdf/en/Neo-fed-up.pdf) |
| Neo iznutra (tehnička knjiga; tema Kristal sa DJ Nick-om je koncept, mogućnost, a ne dogovor) | [PDF](docs/pdf/sr/Neo-tehnicka-knjiga.pdf) | [PDF](docs/pdf/en/Neo-technical-book.pdf) |

Specifikacija je radni nacrt. Sve brojke u njoj su inženjerske procene, a ne merenja.

## Koncept stranice i demoi

Isti dokumenti kao veb stranice, i demoi koji rade u pregledaču. GitHub prikazuje HTML kao izvorni kod, pa preuzmi fasciklu (ili ceo repo kao ZIP) i otvori fajl u pregledaču. Sve je koncept i simulacija, a ne pravi sistem.

| | Srpski | English |
| --- | --- | --- |
| Svi demoi (počni ovde) | [neo-demoi.html](docs/html/sr/demo/neo-demoi.html) | [neo-demos.html](docs/html/en/demo/neo-demos.html) |
| Ormar Pro i Dvojac (menadžer fajlova) | [neo-ormar.html](docs/html/sr/demo/neo-ormar.html) | [neo-cabinet.html](docs/html/en/demo/neo-cabinet.html) |
| Neo DOS (komandna linija) | [neo-dos.html](docs/html/sr/demo/neo-dos.html) | [neo-dos.html](docs/html/en/demo/neo-dos.html) |
| Pločice i Traka | [neo-ploce.html](docs/html/sr/demo/neo-ploce.html) | [neo-tiles.html](docs/html/en/demo/neo-tiles.html) |
| Neo sto, škola, jezici, vesti, Claude pomoć, veza sa telefonom | [docs/html/sr/demo](docs/html/sr/demo) | [docs/html/en/demo](docs/html/en/demo) |
| Pregled, brošura, tehnička knjiga, specifikacija kao veb stranice | [docs/html/sr](docs/html/sr) | [docs/html/en](docs/html/en) |

## Gde rad počinje: Neo OS Lite

Pun NeoOS cilja Neo hardver koji još ne postoji (Neo 500, 2000 i 4000). **Neo OS Lite** je profil za male i postojeće mašine, pa je to deo koji se može graditi i testirati već sada:

- **Neo 100**: mali ARM računar u tastaturi.
- **Neo Revive**: sistem koji se pokreće sa USB-a na starim 64-bitnim PC računarima (UEFI, otprilike od 2012, 2 GB RAM-a ili više) i na pločama klase Raspberry Pi 4/5.
- **Telefoni**: eksperimentalno, samo gde postoje otvoreni drajveri.

Lite dolazi sa Neo radnom površinom. Prvi razvojni cilj je opšti **x86_64 u QEMU-u**, a drugi opšti **AArch64** (QEMU `virt`).

## Arhitektura u jednoj tabeli

| Sloj | Šta je |
| --- | --- |
| Aplikacije i igre | Menadžer fajlova, pregledač, kancelarija, Arkada, programi iz kataloga Riznica |
| Neo Kit i Lux | Jedan alat za prozore, teme, Datatipovi, port za skripte u svakoj aplikaciji |
| Sistemski servisi | Exec-N (portovi za poruke), Dispečer, Hronos (snimci stanja), Obnova (A/B ažuriranja), Brava (privatnost), Čuvar (rezervne kopije), Nađi |
| Drajveri | Izvorni drajveri u Rustu; kod Linux drajvera samo u izolovanim komponentama |
| Tvrđava | Mikrojezgro klase seL4 koje deli memoriju, procesorsko vreme i ovlašćenja |
| Neo ROM i Sef | Pokretač, potpisana A/B slika sistema, ključevi |

Pravila dizajna: odziv pre propusnosti; jedan sistem, jedan API, jedan izgled; svaka aplikacija se može skriptovati; malo i razumljivo, sa budžetom veličine i memorije za svaku sistemsku aplikaciju. Sistem se piše u Rustu.

Izgled radne površine, *Neo Srebro*, prati klasični Mac OS uz najbolje iz BeOS-a, Haiku-a, MorphOS-a i Amige. Svako ime, ikona i crtež su NeoOS-ovi. Srpski (latinica i ćirilica) i engleski (UK i US) su prvi jezici, a komplet za lokalizaciju olakšava dodavanje bilo kog drugog jezika.

## Na temeljima otvorenog koda

Sve što već postoji kao zreo otvoren projekat uzima se kao temelj, ispočetka se piše samo ono što je novo, a izmene se vraćaju izvornim projektima. Kandidati su Genode na seL4, kod Linux drajvera u izolovanim komponentama, Btrfs i EROFS, Mesa, Smithay, Haiku Interface Kit, PipeWire, QEMU, LLVM i Rust.

## Kako pomoći

Kod još nije ovde; u [docs/STATUS.sr.md](docs/STATUS.sr.md) piše kada stiže i koje oblasti se prve otvaraju. Do tada su dobrodošla pitanja, ideje i komentari na specifikaciju kroz issues.

## Licenca

Licenca ovog repozitorijuma još nije konačna. Fajl [LICENSE](LICENSE) za sada sadrži GNU GPL verziju 2.
