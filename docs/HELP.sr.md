# Pomoć i uputstvo: u sistemu, kao HTML i kao PDF

[English](HELP.md) · **Srpski** · Plan, 27. novembar 2026.

Vlasnik je tražio da uputstvo i pomoć budu što dostupniji: **u samom sistemu** kao grafičke poruke i slike interfejsa, i **van njega** kao HTML i PDF.

## Pročitaj vodič odmah

| | Srpski | English |
| --- | --- | --- |
| **Korisnički vodič, početna strana** | [guide/html-sr/index.html](../guide/html-sr/index.html) | [guide/html-en/index.html](../guide/html-en/index.html) |
| Ceo vodič kao jedan PDF | [NBPOS-vodic-sr.pdf](../guide/NBPOS-vodic-sr.pdf) | [NBPOS-guide-en.pdf](../guide/NBPOS-guide-en.pdf) |
| Beta priručnik radne površine | [neo-prirucnik-beta.html](html/sr/neo-prirucnik-beta.html) | [neo-manual-beta.html](html/en/neo-manual-beta.html) |
| Lista mogućnosti | [FEATURES.sr.html](../guide/html-sr/FEATURES.sr.html) | [FEATURES.html](../guide/html-en/FEATURES.html) |
| Igre | [GAMES.sr.html](../guide/html-sr/GAMES.sr.html) | [GAMES.html](../guide/html-en/GAMES.html) |
| Kreativni programi (FilmCraft, PhotoCraft, Album) | [CREATIVE-APPS.sr.html](../guide/html-sr/CREATIVE-APPS.sr.html) | [CREATIVE-APPS.html](../guide/html-en/CREATIVE-APPS.html) |
| Muzika | [MUSIC.sr.html](../guide/html-sr/MUSIC.sr.html) | [MUSIC.html](../guide/html-en/MUSIC.html) |
| Za decu | [KIDS.sr.html](../guide/html-sr/KIDS.sr.html) | [KIDS.html](../guide/html-en/KIDS.html) |
| Pokretačka slika i prvo paljenje | [BUILD-ISO.sr.html](../guide/html-sr/BUILD-ISO.sr.html) | [BUILD-ISO.html](../guide/html-en/BUILD-ISO.html) |
| Demoi sistema u pregledaču | [neo-demoi.html](html/sr/demo/neo-demoi.html) | [neo-demos.html](html/en/demo/neo-demos.html) |

HTML strane se otvaraju iz preuzete kopije repozitorijuma, iz foldera ili sa USB ključa, bez servera i bez mreže. Na GitHub-u prvo preuzmi repozitorijum kao ZIP, jer GitHub prikazuje HTML kao izvorni kod.

## Jedan izvor, tri oblika

Svaka strana pomoći piše se jednom, u Markdown-u, na engleskom i srpskom (`help/en/…`, `help/sr/…`), sa slikama pored. Korak u gradnji pravi sva tri oblika iz tog jednog izvora, pa se ne mogu razilaziti.

| Oblik | Za | Kako |
| --- | --- | --- |
| **U sistemu** | Svi, bez interneta, u trenutku kad zatreba | Program Pomoć i pomoć po kontekstu (ispod) |
| **HTML** | Čitanje u bilo kom pregledaču, na veb stranici, na drugom računaru | `scripts/build-docs.sh html`: jedna strana po temi, sadržaj i polje za pretragu, ne treba server, otvara se iz foldera ili sa USB ključa |
| **PDF** | Štampanje, slanje, čitanje na papiru | `scripts/build-docs.sh pdf`: ceo vodič kao jedna knjiga, sa sadržajem i slikama |

## U sistemu

1. **Taster pomoći.** Jedan taster (F1 i dugme u traci menija) otvara pomoć za ono što je na ekranu: prozor, dijalog, grešku.
2. **Grafičke poruke.** Umesto zida teksta, pomoć pokazuje **sliku pravog interfejsa** sa numerisanom strelicom ili okvirom na delu koji se objašnjava, i dva-tri kratka reda. Slike se snimaju iz pravog sistema probom (ispod), pa ne zastarevaju.
3. **„Pokaži mi”.** Gde je zadatak od nekoliko koraka, Pomoć može da pokaže na pravu kontrolu (okvir pulsira oko nje), korak po korak. Nikad ne klikće umesto korisnika.
4. **Dijana** čita nivo „Nagoveštaj” (kao u [KIDS.sr.md](KIDS.sr.md)) i upućuje na pravu stranu pomoći; ne ponavlja je.
5. **Poruke o grešci** nose NEO kod i vezu do strane pomoći za taj kod. U Lako režimu kod stoji iza „detalji za odrasle”.
6. **Obilazak pri prvom paljenju:** pet kratkih ekrana sa slikama, može da se preskoči, a ponovo je dostupan iz Pomoći.
7. **Jezik.** Pomoć prati jezik sistema i menja se sa njim dok sistem radi ([FEATURES.sr.md](FEATURES.sr.md)). Ako strana nema prevod, prikazuje engleski i to kaže.
8. **Uvek bez interneta.** Pomoć je deo slike sistema. Bez naloga, bez mreže.
9. **Pristupačnost.** Svaka slika ima tekstualni opis; strane rade sa čitačem ekrana i velikim profilom; ništa važno nije samo u slici.

## Tri sloja dubine

- **Brzi početak** (jedan ekran po programu): šta je, tri stvari koje ćete prvo raditi.
- **Korisničko uputstvo** (nekoliko strana po programu): zadaci, sa slikama.
- **Priručnik** (tehnički): prečice, formati, Neo DOS komande. Ovo je deo za iskusne.

## Svaki program ima istu prvu stranu pomoći

„Na čemu je ovaj program zasnovan”, sa istim podacima kao „O programu” ([APP-NAMES.sr.md](APP-NAMES.sr.md)), pa „Kako početi”. Igre počinju sa „Kako se igra” ([GAMES.sr.md](GAMES.sr.md)).

## Slike iz pravog sistema

Proba paljenja pokreće sistem u QEMU-u, otvara svaki program u glavnim stanjima i snima sliku ekrana sa imenom sa spiska (`help/shots.list`). Program Pomoć prikazuje te fajlove, a HTML i PDF gradnja uključuju iste fajlove. Kad se interfejs promeni, slike se promene sa sledećom gradnjom. Danas repozitorijum već ima ovakve snimke ekrana u `docs/img/`.

Strelicu, okvir ili broj dodaje povrh slike mala skripta iz tekstualnog fajla (`help/annotations/…`) sa koordinatama, pa originalni snimak ekrana nikad ne uređuje ručno.

## Početak već danas

Repozitorijum već ima beta priručnik (HTML) i demoe u `docs/html/`, i šest PDF dokumenata. Prvi korak je da se oni prebace u gradnju iz jednog izvora, pa da se doda program Pomoć u sistemu kad radna površina postoji.

## Alat za gradnju

`scripts/build-docs.sh` koristi `pandoc` za HTML i `wkhtmltopdf` za PDF, oba slobodan softver. To je prva verzija i probana je samo na dokumentima iz ovog foldera. `scripts/build-guide.py` gradi objavljeni vodič u `guide/` (HTML za oba jezika sa sadržajem i po jedan PDF za svaki jezik); treba mu samo pandoc, a za PDF wkhtmltopdf ili Chrome bez prozora. Pokreni ga posle svake izmene strane u `docs/`.

## Redosled

1. Gradnja HTML-a i PDF-a iz jednog izvora (urađeno kao prva verzija).
2. Proba sa snimcima ekrana i skripta za oznake.
3. Program Pomoć, F1, kodovi grešaka sa vezama (posle radne površine i Neo Kita).
4. „Pokaži mi” i obilazak pri prvom paljenju.
