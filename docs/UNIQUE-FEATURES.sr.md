# Šta samo NBPOS radi

[English](UNIQUE-FEATURES.md) · **Srpski** · Ažurirano 28. novembra 2026.

Posebna lista stvari koje Neo Boing Platinum OS (NBPOS) radi drugačije od svih današnjih sistema koje znamo: Windows, macOS, Linux radne površine (GNOME, KDE), ChromeOS, Haiku i današnji sistemi nalik Amigi. Uz svaku stavku pošteno piše šta joj je najbliže, da se tvrdnja može proveriti. „Koliko znamo” važi za svaki red: ako znaš sistem koji nešto od ovoga već radi, otvori issue i ispravićemo stranicu.

Sve ovde je **[lite]** ili **[gotovo]** i radi na običnim x86_64 i ARM računarima; ništa ne traži Neo hardver. Jedini izuzetak u listi dokazanih ideja je rukopis (Newton), koji tu stoji samo kao ideja za daleku budućnost.

Oznake: **[gotovo]** radi i testirano je u CI-ju · **[lite]** planirano za Lite.

## Sto i ekrani

| # | Funkcija | Šta je drugačije | Najbliže postojeće | Stanje |
| --- | --- | --- | --- | --- |
| 1 | **Vidžeti kao ekran iza** (Pro režim) | Vidžeti žive na svom sloju između stola i programa. Povučeš program malo nadole i vidžeti su tu; sloj ostaje gde ga pustiš | macOS Dashboard (poseban Space, ukinut 2019), Windows tabla vidžeta koja uklizi sa strane | [lite] |
| 2 | **Amiga povlačenje ekrana na današnjim PC-jevima** | Povučeš program preko celog ekrana nadole za naslovnu traku i vidiš šta je iza; ostaje gde ga ostaviš ([AMIGA.sr.md](AMIGA.sr.md), A10) | AmigaOS i MorphOS na svom hardveru; nijedan uobičajeni PC sistem | [lite] |
| 3 | **Prozori pamte monitor po njegovom identitetu** | Svaki prozor zna monitor po proizvođaču, modelu i serijskom broju, i vraća se na njega kad se monitor ponovo utakne ([MULTI-MONITOR.sr.md](MULTI-MONITOR.sr.md)) | Windows 11 i macOS ponekad pri ponovnom povezivanju, ne po programu | [lite] |
| 4 | **Profili monitora jednim klikom** | „Rad”, „Bioskop”, „Prezentacija”: monitori, rezolucije, Traka i koji program ide na koji monitor, sve jednim klikom | DisplayFusion i slični plaćeni dodatni programi; ništa ugrađeno | [lite] |
| 5 | **Video zid bez posebnog programa** | Mnogo monitora ili projektora kao jedna slika, sa ispravkom okvira, raspoređeno prevlačenjem; kasnije i preko više NBPOS računara | NVIDIA Mosaic i AMD Eyefinity (samo neke kartice), posebni programi za video zidove | [lite] |
| 6 | **Prikaži jedan prozor na drugom monitoru** | Živa kopija samo jednog prozora ide na projektor ili televizor; ostatak tvog stola ostaje privatan | prikaz za predavača unutar pojedinih programa, deljenje prozora u programima za pozive | [lite] |

## Jezik i dokumenti

| # | Funkcija | Šta je drugačije | Najbliže postojeće | Stanje |
| --- | --- | --- | --- | --- |
| 7 | **Otvoreni prevod desnim klikom, bez interneta** | Bilo koji dokument, stranica ili izbor na neki veliki svetski jezik, lokalnim otvorenim alatom (LibreTranslate sa Argos modelima); ništa ne ide u oblak | Firefox lokalni prevod (samo veb stranice), macOS Translate (izbori, Apple modeli) | [lite] |
| 8 | **Jezik se menja dok radiš** | Ceo sistem menja jezik bez ponovnog pokretanja i bez zatvaranja dokumenata; latinica i ćirilica se menjaju odvojeno od jezika | GNOME, KDE i Windows traže odjavu ili ponovno pokretanje | [lite] |

## Karakter i ponašanje

| # | Funkcija | Šta je drugačije | Najbliže postojeće | Stanje |
| --- | --- | --- | --- | --- |
| 9 | **Dijana, stanarka koja nikad ne prekida** | Logo je lik koji živi na Traci, pozdravlja, spava, da savet posle greške i kaže laku noć, ali nikad ne iskače na put ([MASCOT.sr.md](MASCOT.sr.md)) | Clippy (on je prekidao), ljubimci za radnu površinu (nisu deo sistema) | [gotovo] prvi deo |
| 10 | **Tri navike za reset odjednom** | Rade Ctrl+Alt+Del (DOS i Windows), Ctrl+Neo+Esc (Amiga) i oba Shift tastera + Esc za tvrdi reset (RISC OS) | svaki sistem ima samo svoju | [gotovo] |
| 15 | **Smisao za humor koji se može isključiti** | Platinasta loptica, namigivanja Planu 9 i Autostoperskom vodiču, rečenice dana iz naučne fantastike i epske fantastike, haiku od sistema, ime za tvoj računar, tvoj lični moto i Šašavi mod; jedan prekidač ih sve gasi, a prava poruka je uvek prva ([EASTER-EGGS.sr.md](EASTER-EGGS.sr.md)). Inspirisano sistemom **Plan 9 from Bell Labs** (i filmom *Plan 9 from Outer Space*), namerno | `fortune` na Unixu, skrivene igre u starim sistemima | [gotovo] prvi deo (`PLAN9`, `ULLA`, `BOING`, `KNIGHT`, `KOAN`, `HAIKU` u Neo DOS-u) |
| 11 | **Svaki program kaže na čemu je zasnovan** | Jednostavna imena (Album, FilmCraft, PhotoCraft) i, na uvodnom ekranu, u O programu i u Pomoći, otvoreni projekat ispod (gThumb, Kdenlive, GIMP) ([APP-NAMES.sr.md](APP-NAMES.sr.md)) | Linux distribucije pokazuju samo originalna imena | [lite] |

## Ugrađeni servisi

| # | Funkcija | Šta je drugačije | Najbliže postojeće | Stanje |
| --- | --- | --- | --- | --- |
| 12 | **Mreža kao tip podataka** | Prikaz veba, udaljeni folderi (FTP, SFTP, SMB, WebDAV), Flash (Ruffle), Java (`.jar`) i prevod su sistemske usluge koje koristi svaki program, kao što su Amiga tipovi podataka činili format poznatim svakom programu ([NETWORK-DATA.sr.md](NETWORK-DATA.sr.md)) | macOS WebKit i GVfs/KIO na Linuxu pokrivaju po deo; niko ih ne tretira kao jednu porodicu sa jednim spiskom privatnosti | [lite] |
| 13 | **Udaljeni server je ikona diska sa imenom** | `NAS:` ili `POSAO:` na stolu, isto u Ormaru, Neo DOS-u i svakom programu, montiran namerno i naveden kao običan tekst | AmigaOS spisak za montiranje; Linux i macOS montiraju mrežne foldere ali ih kriju u bočnoj traci | [lite] |
| 14 | **Samo drajveri koji tvom hardveru trebaju** | Drajver se pokreće tek kad se nađe njegov uređaj, a uređaj utaknut kasnije tada pokrene svoj drajver; spisak onoga što radi je vidljiv ([NETWORK-DATA.sr.md](NETWORK-DATA.sr.md)) | AmigaOS `DEVS:` i unosi za montiranje; Linux učitava module na zahtev, ali podrazumevana slika nosi hiljade | [gotovo] delimično |

## Dokazane ideje, od 1982. do danas

Nisu nove, i u tome je poenta: svaka je olakšala život na sistemu koji su ljudi voleli, a većina se izgubila kad su ti sistemi nestali. NBPOS ih vraća na običnim računarima. Godina je prvo javno izdanje koje je imalo ideju.

| Godina | Ideja i odakle je | Šta korisnik dobija | U NBPOS-u | Stanje |
| --- | --- | --- | --- | --- |
| 1982 | **Trenutno pokretanje**, ZX Spectrum i Commodore 64 (sistem u ROM-u): spreman za kucanje za oko sekundu | Nema čekanja; računar deluje kao kućni aparat | Cilj: Lite do stola za manje od 5 s sa SSD-a na referentnom računaru; pokreću se samo potrebni drajveri | [lite] |
| 1983 | **Poništi i rad usmeren na dokument**, Apple Lisa | Greška ne košta ništa; otvaraš dokumente, ne programe | Poništi svuda, Blokovi (šabloni), nema dugmeta Sačuvaj ([CLASSIC-MAC.sr.md](CLASSIC-MAC.sr.md)) | [lite] |
| 1985 | **RAM disk koji preživi topli restart**, Amiga `RAM:` i kasnije `RAD:` | Radni fajlovi sačuvani preko restarta | `RAD:` ([RAD.sr.md](RAD.sr.md)) | [lite] |
| 1985 | **Unosi za montiranje i `DEVS:`**, AmigaOS | Svaki uređaj ima ime, pokreće se samo kad se koristi i lako se vidi | `DEVS:`, `MOUNT`, `ASSIGN`, drajver po uređaju ([NETWORK-DATA.sr.md](NETWORK-DATA.sr.md)) | [gotovo] delimično |
| 1987 | **Port za skripte u svakom programu**, ARexx (uz AmigaOS 2.0 od 1990) | Programi rade zajedno: jedna skripta kaže Peru, Ormaru i pošti šta da rade | Lux port u svakom programu, sa komandama u ARexx stilu, iz Neo DOS-a i iz Pera | [lite] |
| 1988 | **Program je folder koji zna kako da se pokrene**, RISC OS `!Run` i `!Boot` | Instaliraš kopiranjem jedne ikone, ukloniš brisanjem; nema instalera ni ostataka | Neo paket programa: jedan folder sa programom, ikonom, tipovima fajlova i skriptom za pokretanje | [lite] |
| 1989 | **Prevuci da sačuvaš**, RISC OS | Čuvaš tako što prevučeš ikonu fajla iz programa pravo u bilo koji folder | Ormar i svaki Neo Kit program ([RISC-OS.sr.md](RISC-OS.sr.md)) | [lite] |
| 1989 | **Meni usluga**, NeXTSTEP | Svaki program nudi svoje radnje (prevedi, pošalji na telefon, napravi PDF) svakom drugom programu | Meni na desni klik koji pune sistemske usluge, prvo Otvoreni prevod | [lite] |
| 1991 | **Alias koji prati original**, klasični Mac System 7 | Veza radi i kad se fajl premesti ili preimenuje | Neo veza, prati fajl po identitetu, ne po putanji | [lite] |
| 1992 | **Tipovi podataka (datatypes)**, AmigaOS 3.0 | Sistem zna tip fajla po sadržaju, ne po ekstenziji; svaki program otvara svaki poznat format | Tipovi podataka, preko 1.000 formata, dekoderi u pesku; i mreža ([NETWORK-DATA.sr.md](NETWORK-DATA.sr.md)) | [lite] |
| 1992 | **Sve je fajl, i mrežni uređaji**, Plan 9 iz Bell Labs | Jedan način da se dođe do diska, servera ili uređaja | Filozofski osnov Neo sistema montiranja: diskovi, serveri i uređaji otvaraju se kroz ista imena | [lite] |
| 1993 | **Prepoznavanje rukopisa**, Apple Newton | Pišeš po ekranu i to postaje tekst | Daleka budućnost, ovde samo pomenuto; nije planirano za Lite | ideja |
| 1997 | **Folderi na oprugu**, Mac OS 8 | Prevučeš fajl na folder, sačekaš, i on se otvori da ideš dublje | Ormar i sto | [lite] |
| 1997 | **Prošireni atributi**, BeOS fajl sistem (BFS), danas u Haiku-u | Ocena, oznake, datum snimanja i autor stoje na samom fajlu i mogu da se pretražuju, bez posebne baze | Atributi na NBPOS diskovima, koriste ih Album, Gramofon, Traži i Ormar ([BEOS-HAIKU.sr.md](BEOS-HAIKU.sr.md)) | [lite] |
| 2007 | **Brz pregled razmaknicom**, Mac OS X 10.5 | Vidiš bilo koji fajl bez otvaranja programa | U Ormaru, kroz tipove podataka | [lite] |

## Kako dodati na ovu listu

Funkcija ide ovde samo kad možemo da navedemo šta joj je najbliže i jasno kažemo šta je drugačije. Ideje su dobrodošle kao issue; detalji svake funkcije ostaju na njenoj stranici ([FEATURES.sr.md](FEATURES.sr.md), [MULTI-MONITOR.sr.md](MULTI-MONITOR.sr.md), [NETWORK-DATA.sr.md](NETWORK-DATA.sr.md), [AMIGA.sr.md](AMIGA.sr.md)).
