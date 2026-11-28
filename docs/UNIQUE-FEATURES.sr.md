# Šta samo NBPOS radi

[English](UNIQUE-FEATURES.md) · **Srpski** · Ažurirano 28. novembra 2026.

Posebna lista stvari koje Neo Boing Platinum OS (NBPOS) radi drugačije od svih današnjih sistema koje znamo: Windows, macOS, Linux radne površine (GNOME, KDE), ChromeOS, Haiku i današnji sistemi nalik Amigi. Uz svaku stavku pošteno piše šta joj je najbliže, da se tvrdnja može proveriti. „Koliko znamo” važi za svaki red: ako znaš sistem koji nešto od ovoga već radi, otvori issue i ispravićemo stranicu.

Sve ovde je **[lite]** ili **[gotovo]** i radi na običnim x86_64 i ARM računarima; ništa ne traži Neo hardver.

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
| 11 | **Svaki program kaže na čemu je zasnovan** | Jednostavna imena (Album, FilmCraft, PhotoCraft) i, na uvodnom ekranu, u O programu i u Pomoći, otvoreni projekat ispod (gThumb, Kdenlive, GIMP) ([APP-NAMES.sr.md](APP-NAMES.sr.md)) | Linux distribucije pokazuju samo originalna imena | [lite] |

## Kako dodati na ovu listu

Funkcija ide ovde samo kad možemo da navedemo šta joj je najbliže i jasno kažemo šta je drugačije. Ideje su dobrodošle kao issue; detalji svake funkcije ostaju na njenoj stranici ([FEATURES.sr.md](FEATURES.sr.md), [MULTI-MONITOR.sr.md](MULTI-MONITOR.sr.md), [AMIGA.sr.md](AMIGA.sr.md)).
