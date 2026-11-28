# DreamCatcher: svaka ideja ima svoje mesto

[English](DREAMCATCHER.md) · **Srpski** · Plan, 28. novembar 2026. · **[lite]**

DreamCatcher (Hvatač snova) je novi NBPOS program: **mapa uma**, **organizator dokumenata** i **pratilac toka rada** u jednom prozoru. Hvata ideje pre nego što odlete, pokazuje kako su dokumenti, ljudi i zadaci povezani i ide sa vama od prve skice do gotovog rada. Slogan: **„Svaka ideja ima svoje mesto.”**

Ime je radno; provera žiga dolazi pre izdanja (nekoliko proizvoda već koristi ime „Dreamcatcher”).

## Šta radi

- **Čvorovi na tabli.** Svaka ideja je čvor: reč, rečenica, slika, glasovna beleška, fajl iz Ormara, dokument iz Pera, veza, zadatak. Čvorove spajaju linije koje kažu kako su povezani („treba mu”, „deo je”, „protivreči”, „sledeće”).
- **Dokumenti su pravi čvorovi.** Prevucite fajl iz Ormara na tablu i on je povezan, ne kopiran; preimenujete ili premestite fajl, a tabla ga i dalje nalazi (preko Traži, lokalnog indeksa). Dvoklik ga otvara u njegovom programu, preko Neo Razvodnika ([PLAN9.sr.md](PLAN9.sr.md)).
- **Od mape do sadržaja do zadataka.** Ista tabla se vidi kao mapa uma, kao sadržaj (koji Pero otvara kao kostur dokumenta) ili kao spisak zadataka u tri kolone: *Za uraditi*, *U toku*, *Gotovo*. Čvor postaje zadatak jednim tasterom.
- **Pratilac toka rada.** Tabla može imati korake („Istraživanje › Nacrt › Pregled › Slanje”). DreamCatcher pokazuje na kom ste koraku, šta je još otvoreno i nudi Blok (šablon) za sledeći korak. Bez napadnih rokova: Dijana pomene otvoren korak najviše jednom dnevno, i to samo ako ste je zamolili.
- **Brzo hvatanje.** Kombinacija tastera odasvud (Neo + D) otvara mali prozor: otkucajte ili izgovorite ideju, ona pada na tablu „Prijemno” i čeka da je rasporedite.
- **Zajedno na lokalnoj mreži.** Dva ili više NBPOS računara u istoj kući ili učionici mogu da otvore jednu tablu i menjaju je u isto vreme, svako sa svojim obojenim pokazivačem, bez naloga i bez servera na internetu (vidi [TOGETHER.sr.md](TOGETHER.sr.md)).
- **Običan format fajla.** Tabla je folder sa jednim tekstualnim fajlom (čvorovi i veze) i njenim slikama, pa je Čuvar, Hronos i `p9fs` tretiraju kao svaki drugi folder; izvoz u OPML, Markdown, FreeMind `.mm`, PNG, SVG i PDF.

## Radi sa

| Program | Kako |
| --- | --- |
| Pero | Sadržaj se otvara kao novi dokument; naslovi iz dokumenta mogu se vratiti kao čvorovi |
| Ormar Pro | Prevlačenje fajlova; tabla može biti „pametni folder” u Ormaru; Magellan pravila mogu stavljati nove fajlove na tablu |
| Rokovnik | Zadatak sa datumom vidi se u kalendaru |
| Traži | Pretražuje table, čvorove i dokumente na koje pokazuju |
| Neo Škola | Gotove table za školske projekte i za učenje jezika |
| Dečja soba | Jednostavan režim sa velikim čvorovima, slikama i nalepnicama, bez kolona zadataka ([KIDS.sr.md](KIDS.sr.md)) |

## Zasnovano na

Mašina za mape uma prati programe otvorenog koda: **Minder** (GPL-3.0) i **VYM** (GPL-2.0) za mapu, **Heimer** (GPL-3.0) za jednostavan stil i FreeMind format za razmenu. Uređivanje uživo preko mreže koristi CRDT biblioteku (**Yjs** ili **Automerge**, obe MIT). Neo interfejs je nov, pisan Neo Kit-om. Licence se ponovo proveravaju pre preuzimanja koda ([APP-NAMES.sr.md](APP-NAMES.sr.md)).

## Procena i redosled

Oko 8 čovek-meseci: tabla i čvorovi 3, veze sa Ormarom, Perom i Traži 2, prikaz sadržaja i zadataka 1,5, uređivanje uživo na lokalnoj mreži 1,5. Prvo mu trebaju Neo Kit i desktop, pa dolazi posle Ormara i Pera. Prva verzija može ranije, kao Flatpak program u Linux kutiji (Minder sa Neo temom), da se ideja isproba.
