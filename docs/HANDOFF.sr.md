# Predaja posla: kako nastaviti kodiranje NBPOS-a

[English](HANDOFF.md) · **Srpski** · 27. novembar 2026.

Za programera, ili AI asistenta sa malim kontekstom (na primer besplatan nalog), koji nastavlja bez ranijih razgovora.

## Čitati ovim redom

1. [README.sr.md](../README.sr.md): šta je NBPOS i šta znači Lite.
2. [STATUS.sr.md](STATUS.sr.md): etape i šta je gotovo.
3. [CONTRIBUTING.sr.md](../CONTRIBUTING.sr.md): građenje, probe, pravila koda, kako se dodaje program ili servis.
4. [decisions/0001-fortress-and-genode.sr.md](decisions/0001-fortress-and-genode.sr.md): zašto će jezgro ustupiti mesto Genode-u na seL4.
5. [FEATURES.sr.md](FEATURES.sr.md): obim, **prvo Lite**, concept art odvojen.
6. Ova strana. [PROGRESS.md](PROGRESS.md) ima oko 50 KB: čitati samo poslednji odeljak.

## Pravila za svaki korak

- Jedan korak, jedna grana, nazvana `claude/neo-lite-stepNN-kratak-naziv`, jedan pull request.
- Probe paljenja moraju da prolaze: `./scripts/test-boot-x86_64.sh`, `./scripts/test-boot-aarch64.sh`, `./scripts/test-usb-x86_64.sh`. Kad se doda program, ažurirati brojeve fajlova u skriptama.
- Formatirati sa `cargo fmt` u korenu i u `user/`.
- Na kraju koraka dodati zapis u PROGRESS.md i štiklirati liniju u STATUS.md.
- Dokumenti idu na engleskom i srpskom (`X.md` i `X.sr.md`), srpski latinicom.
- Kod je GPLv3 ili novija. Šta je pod drugom licencom ide kao poseban program (pravilo u [AMIGA.sr.md](AMIGA.sr.md)).
- Samo sopstvena imena, ikone i crteži: nijedno tuđe ili licencirano ime u sistemu.
- Concept art svuda označiti kao koncept (dokumenti, video, primeri).
- Strane o sistemu govore samo o x86_64 i ARM-u (referentne mašine: Raspberry Pi 5, AMD Zen 2, Intel Core 10. gen.). Neo hardver (Neo 500, 2000, 4000, NeoPocket, čipovi, FPGA) ide samo u kratak odeljak „Konceptni hardver” na kraju strane.
- Claude integracija se uvek označava „(daleka budućnost, nije garantovano)”; nikad je ne obećavati.
- Ne tvrditi da mogućnost radi dok je proba ne pokaže.
- Svaki tekst koji korisnik vidi dolazi iz jezičkog kataloga, nikad iz ugrađenog niza, da bi jezik mogao da se menja dok sistem radi ([FEATURES.sr.md](FEATURES.sr.md)).

## Red čekanja (ovim redom)

1. **Prvo paljenje na pravim PC-jevima**, počev od referentnog računara: [BUILD-ISO.sr.md](BUILD-ISO.sr.md), odeljci 4 i 5. Traži vlasnikove ruke; asistent ispravlja ono što pokažu fotografije i logovi.
2. **Genode na seL4:** buđenje signalom, deljena memorija, ručke u porukama; zatim Neo DOS kao Genode komponenta.
3. **Radna površina:** kompozitor, pločice (Sat, Kalkulator, Merač), Polica i Traka, prevlačenje, Neo meni, komandna paleta, pa Neo Srebro.
4. **Fajlovi:** Ormar sa dva panela, arhive, Btrfs i EROFS; `RAD:` nivo 1 ([RAD.sr.md](RAD.sr.md)).
5. **Drajveri za svakodnevnu upotrebu:** skladište (NVMe, AHCI), mreža, zvuk. Sve ispod njih traži.
6. **Linux box** (domen Aplikacije). Trebaju mu foto viewer, HippoPlayer, qBittorrent i Hydrogen.
7. **Pregledač slika, FilmCraft i PhotoCraft** ([CREATIVE-APPS.sr.md](CREATIVE-APPS.sr.md)). Album je gThumb, FilmCraft je Kdenlive, PhotoCraft je GIMP i Inkscape.
8. **Igre** ([GAMES.sr.md](GAMES.sr.md)), počev od Pasijansa i Minolovca. **Pomoć** ([HELP.sr.md](HELP.sr.md)): gradnja HTML/PDF iz jednog izvora postoji; proba sa snimcima ekrana i program Pomoć u sistemu slede posle radne površine.
9. **Muzika i programi** redom iz [MUSIC.sr.md](MUSIC.sr.md).
10. **Dečja verzija** ([KIDS.sr.md](KIDS.sr.md)) kad postoje radna površina, Lako raspored i Ključevi u Bravi, i **pravilo o imenima i zaslugama programa** ([APP-NAMES.sr.md](APP-NAMES.sr.md)) od prvog programa.
11. **Instaler** pored Windowsa 11 ([BUILD-ISO.sr.md](BUILD-ISO.sr.md), odeljak 6).
12. Strane plana redom kako svaka kaže: [AMIGA.sr.md](AMIGA.sr.md), [CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md), [BEOS-HAIKU.sr.md](BEOS-HAIKU.sr.md), [RISC-OS.sr.md](RISC-OS.sr.md).

## Otvoreni izbori koji pripadaju vlasniku

Stavke u `RISC-OS.sr.md` označene kao **nove** (BeOS i Haiku stavke H9 do H11 su prihvaćene); razvrstavanje dokumenata na sistem i koncept ([CONCEPT-COMPUTER.sr.md](CONCEPT-COMPUTER.sr.md)), model za beatmaker, kanonski izgled Dijane ([MASCOT.sr.md](MASCOT.sr.md)) i profil za brz build.

## Nalepiti na početku sesije

> Nastavljaš NBPOS (Neo Boing Platinum OS), prvo Lite. Pročitaj README.sr.md, docs/STATUS.sr.md, CONTRIBUTING.sr.md i docs/HANDOFF.sr.md. Radi na prvoj neoznačenoj stavci reda čekanja iz HANDOFF.sr.md, jedna grana i jedan korak. Drži probe paljenja zelenim, ažuriraj PROGRESS.md i STATUS.md, piši dokumente na engleskom i srpskom i jasno reci šta nisi probao.
