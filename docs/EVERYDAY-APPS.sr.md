# Svakodnevni programi: dobre ideje programa koje ljudi vole

[English](EVERYDAY-APPS.md) · **Srpski** · Plan, 28. novembar 2026. · **[lite]**

Vlasnik je nabrojao programe koje ljudi vole i tražio njihove najbolje ideje u NBPOS-u. Pravilo: uzimamo **ideju**, ne izgled ni ime, gradimo je Neo Kit-om na slobodnom softveru i zadržavamo **prvo bez interneta, bez naloga, bez obaveznog oblaka**. Svaki program ima jednostavno ime i red „zasnovano na” ([APP-NAMES.sr.md](APP-NAMES.sr.md)).

| Uzor | Ideja koju uzimamo | U NBPOS ide kao | Zasnovano na (licencu proveriti) |
| --- | --- | --- | --- |
| **Google Keep** | Beleška uhvaćena za dve sekunde, boje, liste za štikliranje, beleška zakačena na radni sto | **DreamCatcher beleške**: Neo + D hvata belešku; boja, lista, podsetnik; zakačena beleška se vidi kao vidžet na radnom stolu (ili na ekranu vidžeta iza, Pro režim) | DreamCatcher ([DREAMCATCHER.sr.md](DREAMCATCHER.sr.md)) |
| **Obsidian** | Beleške kao obični fajlovi, `[[veze]]` među njima, graf veza | **DreamCatcher režim grafa** plus Pero: folder Markdown beleški, `[[veze]]` u Peru, a DreamCatcher crta mapu veza; ništa zaključano u bazi | Običan Markdown; prikaz grafa u Neo Kit-u |
| **Shortcut** (praćenje projekata) | Priče, tabla, male iteracije, bez težine Jire | **DreamCatcher zadaci**: lak pratilac u tabli: Za uraditi / U toku / Gotovo, oznake, ko radi, rok; za porodicu, razred ili mali tim, ne za firmu | DreamCatcher |
| **Google Classroom** | Zadatak sa rokom, diskusija, fajlovi po tipu | **Školska klupa** (EN: SchoolDesk): nastavnikov NBPOS računar drži razred na lokalnoj mreži; zadaci, rokovi, diskusija po zadatku, fajlovi po tipu (dokumenti, slike, DreamCatcher table); bez Google naloga, ništa ne izlazi iz škole ako škola to ne želi | Neo sopstveni, sa idejama i formatima razmene iz **Kolibri** (MIT, učenje bez interneta) i **Moodle** (GPL-3.0, uvoz i izvoz kurseva) |
| **Dub Music Player** | Čista biblioteka bez gomile, žanrovske „kante”, trenutna pretraga | **Gramofon**, prikaz biblioteke: jedna uredna lista, žanrovi kao kante u koje ubacujete albume, pretraga dok kucate, bez prodavnice i bez nuđenja strimovanja | Gramofon ([MUSIC.sr.md](MUSIC.sr.md)), podaci iz **MusicBrainz** (CC0) |
| **Sonique 2** | Plejer sa vizualizacijama i skinovima | **Gramofon**, prikaz plejera: vizualizacije preko celog ekrana (projectM, LGPL-2.1, radi sa MilkDrop presetima) i skinovi preko Neo sistema tema; jedan mali okrugli skin „kao Sonique” kao omaž | Gramofon, projectM |
| **AccuWeather** | Jasna prognoza, animacija radara | **Vreme** (EN: Weather): vidžet na stolu i poseban program; prognoza po satima i za 14 dana, animacija radara, poslednja prognoza sačuvana za rad bez interneta; lokacija ne izlazi sa računara, osim grada koji ste otkucali | **Open-Meteo** (podaci CC BY 4.0, server AGPL-3.0, može na sopstvenom računaru), radar od nacionalnih službi gde uslovi dozvoljavaju (za Srbiju RHMZ), **RainViewer** po izboru |
| **Discogs i IMDb** | Katalog sopstvenih ploča i filmova, zasluge, detalji izdanja | **Kolekcija** (EN: Collection): lokalna SQLite baza vaše muzike, filmova, knjiga i igara, skenirana iz foldera ili ukucana; prvo bez interneta; dopuna podataka iz MusicBrainz-a (CC0) i, ako ih korisnik uključi, Discogs API-ja i TMDB-a | SQLite (javno dobro), MusicBrainz; Discogs i TMDB samo kao izborni izvori na mreži, po njihovim uslovima |
| **PDFGear** | Besplatan PDF alat: čitanje, beleške, spajanje, deljenje, bez oblaka | **Listaj**, PDF alati: označavanje, beleške, crtanje, formulari, potpisi; spajanje, deljenje, preraspoređivanje, sažimanje; sve na računaru | Poppler (GPL-2.0+), **qpdf** (Apache-2.0), ideje iz **Xournal++** (GPL-2.0+) |
| **Icecream Ebook Reader** | Polica sa knjigama, obeleživači, napredak čitanja | **Listaj**, knjige: polica za EPUB, MOBI, FB2, CBZ i PDF, obeleživači i beleške, napredak čitanja, dnevni cilj po želji (Dijana ga pomene najviše jednom dnevno, samo ako je zamoljena) | **Foliate** (GPL-3.0) |
| **HandBrake** | Konverzija videa u grupama, sa gotovim presetima, čist interfejs | **VideoCraft**: konvertor pored FilmCraft-a: ubacite fajlove, izaberete preset („za telefon”, „za TV”, „malo za poštu”, „arhiva”), pokrenete grupu; koristi grafičku karticu kad može | **HandBrake** (GPL-2.0), FFmpeg |

## Procena i redosled

Ukupno oko 20 čovek-meseci: DreamCatcher beleške, režim grafa i zadaci 4 (povrh 8 za sam DreamCatcher), Školska klupa 5, biblioteka i vizualizacije u Gramofonu 2, Vreme 1,5, Kolekcija 2,5, PDF alati i polica u Listaj 3, VideoCraft 2 (prvo kao HandBrake u Flatpaku, oko 0,2).

1. Prvo Flatpak, čim Linux kutija proradi: HandBrake (VideoCraft), Foliate (knjige u Listaj), projectM.
2. Sa desktopom i Neo Kit-om: vidžet Vreme, DreamCatcher beleške, PDF alati u Listaj.
3. Posle DreamCatcher-a i Pera: režim grafa i zadaci; zatim Školska klupa (treba joj sloj „zajedno” iz [TOGETHER.sr.md](TOGETHER.sr.md)); Kolekcija.
