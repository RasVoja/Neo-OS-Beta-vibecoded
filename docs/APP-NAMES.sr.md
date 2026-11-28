# Imena programa i zasluge: na čemu je koji program zasnovan

[English](APP-NAMES.md) · **Srpski** · Pravilo, 27. novembar 2026.

## Pravilo

1. **Jednostavna imena.** Program nosi kratko, svakodnevno ime (Pero, Ormar, Album), ne ime izvornog projekta.
2. **Reći na čemu je zasnovan.** Gde god se program predstavlja, navodi se i njegova osnova:
   - **Splash ekran:** Neo ime i jedan sitan red, na primer „zasnovano na LibreOffice Writer-u”.
   - **O programu:** osnovni program i njegova verzija, licenca, veza do izvornog koda i šta je Neo promenio.
   - **Pomoć:** prva strana je „Na čemu je ovaj program zasnovan”, sa istim podacima i zahvalnošću autorima.
   - **Dokumentacija i stavka u Riznici:** isti red, istim rečima.
3. **Tuđi programi koji se ne grade iznova zadržavaju svoje ime** (na primer Wesnoth, Hydrogen). I oni dobijaju red „zasnovano na”, napisan kao „napravio”.
4. Zasluge stoje u jednom fajlu po programu (`CREDITS`), a sva četiri mesta ga čitaju, pa se ne mogu razilaziti.
5. Program pod drugom licencom je poseban program (vidi [AMIGA.sr.md](AMIGA.sr.md)); tekst njegove licence prikazuje se u „O programu”.

## Imena

Preuzeto iz specifikacije; redovi od FilmCraft-a naniže su dopune iz novembra 2026.

| Neo ime | Radi | Zasnovano na |
| --- | --- | --- |
| Ormar | fajlovi | Haiku Tracker, libarchive, GVfs |
| Vidik | veb | WPE WebKit, delovi Epiphany-ja |
| Golub | pošta | jezgro Geary-ja |
| Pero | tekst | LibreOffice Writer |
| Abakus | tabele | LibreOffice Calc |
| Platno, Pozornica | crtanje, prezentacije | LibreOffice Draw, Impress |
| Listaj | PDF, EPUB, MOBI, stripovi; beleške u PDF-u, spajanje i deljenje; polica | Poppler, Foliate, qpdf |
| Album | pregled i organizacija slika | gThumb, sa Neo Kit interfejsom |
| Gramofon | muzika: čista biblioteka sa žanrovskim kantama, vizualizacije i skinovi | GStreamer, HippoPlayer, projectM (vidi [MUSIC.sr.md](MUSIC.sr.md)) |
| Bioskop | video | mpv |
| Talas | audio editor | motor Audacity-ja |
| Rokovnik | beleške, kalendar | Evolution Data Server, CalDAV |
| Ključar | lozinke | KeePassXC format |
| Arkada | igre, retro | RetroArch, libretro, MAME, DOSBox, ScummVM, FS-UAE |
| Neo BASIC | prvi programi | jezgro SmallBASIC-a |
| Čuvar | rezervne kopije | Btrfs snimci, restic |
| FilmCraft | video montaža | Kdenlive (kasnije Neo Kit interfejs) |
| PhotoCraft | obrada slika i crtanje | GIMP i Inkscape (kasnije Neo Kit interfejs) |
| Treker | muzika u modulima | libopenmpt, uz ideje iz Schism Trackera i MilkyTrackera |
| Bubnjevi | bubnjarska mašina | Hydrogen |
| Torrent | preuzimanja | qBittorrent |
| Karting, Carstva, Kocke, Mehurići, Wesnoth | igre | SuperTuxKart, 0 A.D., Luanti, Frozen Bubble, Battle for Wesnoth (vidi [GAMES.sr.md](GAMES.sr.md)) |
| DreamCatcher | mapa uma, organizator dokumenata, pratilac toka rada | Ideje iz Minder-a, VYM-a i Heimer-a, FreeMind format, Yjs ili Automerge za uređivanje uživo (vidi [DREAMCATCHER.sr.md](DREAMCATCHER.sr.md)) |
| Kanal | YouTube i PeerTube bez reklama i naloga | FreeTube, kasnije yt-dlp i mpv (vidi [TOGETHER.sr.md](TOGETHER.sr.md)) |
| Sastanak | video pozivi, učionice | Jitsi Meet |
| Emisija | snimanje ekrana, strimovanje | OBS Studio |
| DJ pult | DJ miksovanje | Mixxx |
| Pero zajedno | jedan dokument koji piše više ljudi na lokalnoj mreži | Pero sa CRDT slojem; Collabora Online za pune kancelarijske dokumente |
| Neo Razvodnik | šalje fajl, URL ili adresu pravom programu | plumber iz Plana 9 (vidi [PLAN9.sr.md](PLAN9.sr.md)) |
| Školsko zvono | školska avantura | Originalno, omaž igri *Skool Daze* (ZX Spectrum) |
| Spectrum kutija | emulator ZX Spectruma u Arkadi | Fuse |
| Školska klupa | zadaci za razred, rokovi, diskusija | Neo sopstveni, ideje iz Kolibri i Moodle (vidi [EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md)) |
| Vreme | vidžet i program za prognozu, radar | Open-Meteo, nacionalne meteorološke službe |
| Kolekcija | katalog vaše muzike, filmova, knjiga, igara | SQLite, MusicBrainz; Discogs i TMDB po izboru |
| VideoCraft | konvertor videa, grupe i preseti | HandBrake, FFmpeg |

Srpska imena sistemskih programa (Ormar, Pero, Abakus i tako dalje) ostaju kao u specifikaciji.
