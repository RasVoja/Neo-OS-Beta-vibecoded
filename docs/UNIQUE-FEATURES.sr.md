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
| 12 | **Pametno spuštanje** (Smart Drop) | Dok vučeš, sistem zna tip objekta (preko Datatypes), a odredište **pre nego što pustiš** pokaže šta će uraditi sa njim („Album: dodaj 3 slike”, „Golub: priloži”, „DreamCatcher: beleška sa vezom”). Pretvara u letu kad odredištu treba drugi tip (TIFF spušten na veb formular postaje JPEG, URL spušten na DreamCatcher postaje beleška sa vezom). **Polica** na ivici ekrana čuva fajlove između programa. Modifikatori su svuda isti: bez = premesti, Ctrl = kopiraj, Alt = veza (alias), Shift = pitaj. Ikone programa na Traci su odredišta za spuštanje, kao na Amigi i klasičnom Mac-u | macOS vučenje sa Option i Command, police drugih proizvođača (Yoink, Dropover), KDE meni „Premesti / Kopiraj / Poveži” pri spuštanju; nijedno ne pokazuje unapred šta će odredište uraditi niti pretvara usput | [lite] |

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

## Pristupačnost (obećanje, ne tvrdnja o jedinstvenosti)

Većinu ovoga imaju i drugi sistemi; ovde je zato što ga je vlasnik tražio kao čvrst deo plana, a ne kao dodatak. Koraci kodiranja 150 do 159 u [PROGRESS.md](PROGRESS.md). Sve je **[lite]**.

| Funkcija | Šta NBPOS radi | Stanje |
| --- | --- | --- |
| **Tema visokog kontrasta, ugrađena** | Svetla i tamna tema visokog kontrasta u sistemu tema od prvog izdanja desktopa, i za Neo DOS i ekrane pokretanja; palete bezbedne za daltoniste za Traku, pločice i grafikone | [lite], koraci 150–151 |
| **Rad samo tastaturom** | Svaki prozor, meni, Traka, Neo meni i svaka Neo Kit kontrola mogu se dosegnuti i koristiti bez miša; vidljiv okvir fokusa svuda; lepljivi tasteri, spori tasteri i miš sa tastature | [lite], koraci 152–153 |
| **Čitač ekrana (AT-SPI2)** | Neo Kit daje svakoj kontroli ime i ulogu preko **AT-SPI2**, iste magistrale pristupačnosti koju koriste Linux desktopi, pa **Orca** (GPL-2.0+) čita i Neo programe i programe iz Linux kutije; i ispis Neo DOS-a može da se izgovori | [lite], koraci 154–156 |
| **Skalabilna slova i interfejs** | Ništa nije vezano za piksele: interfejs se uvećava od 100 do 300 odsto, slova odvojeno od njega; vektorske ikone; veliki profil iz specifikacije to koristi | [lite], koraci 157–158 |
| **Mirno i čitljivo** | Manje pokreta poštovano svuda (već u planu za „povuci ekran”), zvučna upozorenja prikazana i na ekranu, titlovi za Dijanu i za video u pomoći, po želji slova prilagođena disleksiji (OpenDyslexic, OFL) | [lite], korak 159 |

## Za generaciju Z i one posle nje

Ono što mladi očekuju od računara, urađeno bez naloga, reklama i oblaka koji nije njihov. Većina toga postoji negde drugde, deo po deo; zajedno, lokalno i sa jednim jednostavnim licem, to je razlika NBPOS-a. Sve je **[lite]**.

| Žele | NBPOS daje | Strana |
| --- | --- | --- |
| Video pozive sa drugarima i razredom | **Sastanak** (Jitsi Meet), dovoljan je link, bez naloga | [TOGETHER.sr.md](TOGETHER.sr.md) |
| YouTube bez reklama i praćenja | **Kanal** (FreeTube), sa PeerTube-om u istoj listi | [TOGETHER.sr.md](TOGETHER.sr.md) |
| Strimovanje svojih igara, muzike, časova | **Emisija** (OBS Studio), **DJ pult** (Mixxx) | [TOGETHER.sr.md](TOGETHER.sr.md) |
| Beleške i planiranje kao u Notion-u i Obsidian-u | **DreamCatcher**: mapa uma, povezane beleške, brze beleške, laka tabla zadataka | [DREAMCATCHER.sr.md](DREAMCATCHER.sr.md), [EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md) |
| Školski rad kao u Google Classroom-u | **Školska klupa**, na sopstvenoj mreži škole | [EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md) |
| Deljenje sa telefonom i drugarima bez oblaka | **LocalSend** i **Syncthing** (već u veži sa telefonom iz specifikacije), KDE Connect | [FEATURES.sr.md](FEATURES.sr.md) |
| Muziku koja lepo izgleda | **Gramofon** sa vizualizacijama i skinovima | [EVERYDAY-APPS.sr.md](EVERYDAY-APPS.sr.md) |
| Bez čekanja | Cilj: **ispod 5 sekundi** od pokretača do radnog stola na NVMe SSD-u (današnja procena za referentni PC je 5–8 s, [FEATURES.sr.md](FEATURES.sr.md) odeljak 4; meri se na pravim PC-jima) | [REAL-PC-TEST.sr.md](REAL-PC-TEST.sr.md) |

## Kako dodati na ovu listu

Funkcija ide ovde samo kad možemo da navedemo šta joj je najbliže i jasno kažemo šta je drugačije. Ideje su dobrodošle kao issue; detalji svake funkcije ostaju na njenoj stranici ([FEATURES.sr.md](FEATURES.sr.md), [MULTI-MONITOR.sr.md](MULTI-MONITOR.sr.md), [AMIGA.sr.md](AMIGA.sr.md)).
