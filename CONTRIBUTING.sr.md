# Kako doprineti Neo OS Lite-u

*English: [CONTRIBUTING.md](CONTRIBUTING.md)*

Hvala što želiš da pomogneš. Ovo uputstvo objašnjava kako se sistem gradi i testira, kako je kod raspoređen i kako se dodaje program ili servis. Opisuje kod kakav je danas: **Tvrđava (Fortress)**, malo početno jezgro napisano u Rustu. Ono se podiže na x86_64 (PC) i na AArch64 (QEMU-ova mašina `virt`) i pokreće programe u korisničkom režimu. Šta je koji korak dodao piše u [docs/PROGRESS.md](docs/PROGRESS.md) (na engleskom).

## Šta ti treba

- **Rust** preko `rustup`-a. `rust-toolchain.toml` bira stabilni kanal i oba cilja bez operativnog sistema (`x86_64-unknown-none`, `aarch64-unknown-none`), a `rustup` ih instalira pri prvom građenju.
- **QEMU**: `qemu-system-x86` i `qemu-system-arm` (imena paketa na Debianu i Ubuntuu).
- **Python 3** za male pomoćne skripte. `scripts/make-font.py` traži još Pillow i font DejaVu Sans Mono, ali samo ako menjaš font.
- Unix školjka. CI koristi Linux, a macOS bi trebalo da radi.

## Građenje i testiranje

```sh
./scripts/test-boot-x86_64.sh    # gradi sve, podiže sistem 5 puta, proverava izlaz
./scripts/test-boot-aarch64.sh   # isto na AArch64, 7 podizanja
```

Svaka skripta gradi jezgro i korisničke programe. Zatim podiže QEMU sa različitim komandnim linijama jezgra: samoproveru, namerne padove i Neo DOS sesije otkucane preko QEMU-ovog monitora. Proverava svaki red koji očekuje i završava se sa `BOOT TEST PASSED`. CI (`.github/workflows/boot.yml`) pokreće obe skripte na svaki push i pull request, a proverava i formatiranje.

Da sam isprobaš sistem:

```sh
./scripts/run-x86_64.sh neo.shell   # Neo DOS na serijskoj konzoli; iz QEMU-a se izlazi sa Ctrl-A, X
```

Neo DOS sa slikom za podizanje i ekranom pokreće se sa istim QEMU opcijama koje koriste test skripte (`-initrd`, a na AArch64 i `-device ramfb`). Umesto `-display none` otvori QEMU prozor, pa ćeš videti radnu površinu i moći da koristiš miš.

### Slika za podizanje na PC-ju

```sh
./scripts/make-usb-image.sh        # target/neo-lite-x86_64.iso, za PC sa BIOS-om i UEFI-jem
./scripts/test-usb-x86_64.sh       # podiže tu sliku sa BIOS-om i sa UEFI-jem i pokreće samoproveru
```

Slika je hibridni ISO: GRUB učitava Tvrđavu preko Multiboot2, a sliku za podizanje kao modul. Podigni je u QEMU-u (`qemu-system-x86_64 -machine q35 -m 512M -cdrom target/neo-lite-x86_64.iso`) ili je upiši na USB fleš. Prva stavka menija pokreće radnu površinu i Neo DOS, druga samoproveru. Potrebni su `grub-pc-bin`, `grub-efi-amd64-bin`, `xorriso`, `mtools` i, za UEFI test, `ovmf`. Na pravom PC-ju radna površina se pojavljuje na ekranu koji podesi GRUB (UEFI GOP ili VBE); kucanje i miš rade sa USB tastaturom i mišem uključenim direktno u PC (xHCI, boot protokol) ili sa PS/2.

Reči na komandnoj liniji jezgra:

| Reč | Šta radi |
| --- | --- |
| `neo.test` | Pokreće sve samoprovere, pa izlazi iz QEMU-a sa statusom 33 ako su sve prošle |
| `neo.shell` | Pokreće servise za ekran i tastaturu i Neo DOS |
| `neo.test=pagefault`, `=stackoverflow`, `=undefined` | Namerno ruši sistem, da se proveri izveštaj o padu |

## Kako je kod raspoređen

| Putanja | Šta je tu |
| --- | --- |
| `fortress/src/main.rs` | Zajednički početak: spisak modula, `run_shell`, rukovalac panike |
| `fortress/src/start_x86_64.rs`, `start_aarch64.rs` | Put podizanja i spisak samoprovera za svaku arhitekturu |
| `fortress/src/arch/` | Procesor, prekidi, tajmer i tabele stranica po arhitekturi |
| `fortress/src/mem/` | Deljenje okvira memorije i hip jezgra |
| `fortress/src/sched.rs`, `ipc.rs` | Niti, spavanje i parkiranje; portovi za poruke |
| `fortress/src/process.rs`, `syscall.rs` | Programi, njihove sposobnosti (handle-ovi) i sistemski pozivi; tabela poziva je na vrhu `syscall.rs` |
| `fortress/src/irq.rs`, `input.rs`, `pointer.rs` | Prekidi predati drajverima u korisničkom režimu, otkucani unos, pokazivač miša |
| `fortress/src/display.rs`, `virtio_*.rs`, `bootfs.rs`, `diskfs.rs` | Uređaji i sistemi datoteka |
| `fortress/src/*_tests.rs` | Samoprovere zajedničke za obe arhitekture |
| `user/` | Korisnički programi (poseban Cargo workspace), sa `neo-rt`, bibliotekom za izvršavanje, `neo-gfx`, za crtanje, tekst i prozore, `neo-keys`, za rasporede tastature, i `neo-ring`, za prstenove u deljenoj memoriji za velike poruke (ta dva koristi i jezgro) |
| `scripts/` | Skripte za građenje, podizanje i testiranje |

`build.rs` jezgra gradi workspace `user/` i ugrađuje programe koji trebaju samoproverama. `scripts/make-bootfs.sh` pakuje ostale u sliku za podizanje (`-initrd`) i na disk.

## Pravila koja izmena mora da poštuje

1. **Obe test skripte prolaze.** Ako izmena menja ono što podizanje ispisuje, u istom commit-u ažuriraj očekivane redove.
2. **Bez upozorenja.** Građenje je bez upozorenja; neka tako i ostane.
3. **Formatirano sa `cargo fmt`.** `rustfmt.toml` dozvoljava redove do 120 znakova. Pokreni `cargo fmt` u korenu i u `user/`.
4. **Ništa ne curi.** Svaka samoprovera koja pokrene program proverava da se broj okvira memorije i broj niti vrate na staro. Tako radi i u novim testovima.
5. **Drajveri žive u korisničkom režimu kad god mogu.** Drajver dobija tačno one sposobnosti koje mu trebaju (portove, liniju prekida, memoriju uređaja) i ništa više. Drajveri u jezgru (`virtio_blk`, `virtio_input`) su privremeni, dok DMA baferi ne počnu da se dele kao sposobnosti.
6. **Komentari kažu zašto.** Svaka datoteka počinje pasusom `//!` o tome čemu služi. Kod i komentari su na engleskom, jednostavnom, da ga početnik prati.
7. **Jedan korak po pull request-u,** uz kratak unos u `docs/PROGRESS.md`: šta se promenilo i kako je testirano.

## Dodavanje korisničkog programa

1. Kopiraj `user/hello-rust` u `user/<ime>` i promeni ime paketa u njegovom `Cargo.toml`.
2. Dodaj `<ime>` u `members` u `user/Cargo.toml`.
3. Napiši `neo_main() -> i64`. Za prozor koristi `neo_gfx::Window::open` (vidi `user/hello-window`). Vrednost koju vrati je izlazni kod. `neo-rt` daje `print`, portove, datoteke, `sleep` i ostalo. Handle-ovi su numerisani od 0, redom kojim ih je jezgro ili roditelj predao.
4. Da bi ušao u sliku, dodaj ga u spisak programa u `scripts/make-bootfs.sh`. Taj spisak je zajednički za obe arhitekture, a postoji i poseban spisak samo za x86_64. Zatim ažuriraj broj datoteka koji test skripte očekuju (`N files`).
5. Neo DOS ga onda pokreće sa `RUN <ime>`.

## Dodavanje servisa

Servis je program koji jezgro pokrene sa sposobnostima. `display.rs` je najkraći potpun primer:

- **Sposobnosti.** Napravi njihov spisak (`Cap::Mem`, `Cap::io`, `Cap::Irq`, port sa `RIGHT_RECV` kao kontrolni port, i tako dalje).
- **Pokretanje.** Pokreni ga sa `process::spawn_elf` i sačuvaj port, da bi jezgro moglo da mu se obrati ili da ga zaustavi.
- **Samoprovera.** Napiši je: pokreni servis, spolja proveri šta radi (piksele, bajtove, poruke), zaustavi ga i proveri da ništa nije iscurelo. Dodaj je u spisak samoprovera svake arhitekture na kojoj servis radi.
- **Očekivani red.** Red koji test ispisuje dodaj u test skriptu.

## Dodavanje sistemskog poziva

1. Uzmi sledeći slobodan broj. Dodaj ga u `match` u `syscall_dispatch` i u tabelu na vrhu `syscall.rs`.
2. Prvo proveri vrstu sposobnosti iza handle-a. Pogrešna vrsta vraća `ERR_DENIED`, a nepostojeći handle `ERR_BAD_HANDLE`.
3. Dodaj omotač u `neo-rt`, sa dokumentacionim komentarom.
4. Testiraj ga kroz program, ne samo iz jezgra.

## Odakle početi

Plan je u `docs/PROGRESS.md`, pod "Next steps". Dvanaest prvih zadataka, svaki sa svojim testom, je u [docs/STARTER-TASKS.sr.md](docs/STARTER-TASKS.sr.md). Neki od njih:

- Dopunjavanje imena fajlova tasterom Tab u Neo DOS-u, ili nov raspored u `user/neo-keys`.
- Nove Neo DOS komande u `user/neodos`, na primer `TIME` ili `MEM`.
- Više znakova u fontu (`scripts/make-font.py`), na primer srpska ćirilica.
- Još jedan test za putanju greške koja još nije pokrivena.

## Licenca

Ovaj i javni repozitorijum su pod GNU GPL verzijom 3 ili kasnijom (vidi `LICENSE`). Doprinosom pristaješ da tvoj doprinos bude objavljen pod tom licencom.
