# Više monitora i video zidovi

[English](MULTI-MONITOR.md) · **Srpski** · Plan, 28. novembar 2026.

Kako Neo Boing Platinum OS (NBPOS) Lite planira da radi sa dva ili više monitora, i sa video zidom od mnogo ekrana. Sve ovde je **[lite]**: radi na običnim x86_64 i ARM računarima i ne traži Neo hardver. Ništa sa ove stranice još ne radi; ovo je plan koji kompozitor prati.

## Problem danas

Većina sistema daje dva izbora: **kloniraj** (svi monitori prikazuju isto) ili **proširi** (jedan širok sto). Posle toga si prepušten sebi:

- Prozori se otvaraju na monitoru koji sistem izabere, a kad izvučeš monitor i vratiš ga, često ostanu nagomilani na prvom.
- Nema jednostavnog načina da kažeš „ovaj program se uvek otvara na desnom monitoru” ili „ovaj prozor preko celog ekrana na televizoru”.
- Drugačiji raspored za drugačije prilike (rad, film, prezentacija) znači da svaki put otvaraš podešavanja ekrana i sve pomeraš rukom.
- Video zid (mnogo ekrana kao jedna slika) traži poseban režim grafičke kartice (NVIDIA Mosaic, AMD Eyefinity) ili plaćeni program.

## Kako to radi NBPOS

### 1. Jedan neprekidan sto

Svi monitori čine jedan sto. Prevučeš prozor sa jednog monitora na drugi i on prirodno nastavi preko ivice, u pravoj veličini i kad monitori imaju različitu rezoluciju ili uvećanje (4K monitor pored 1080p: prozor zadržava veličinu u centimetrima, ne u pikselima). Pokazivač ne zapinje na uglovima gde se ekrani ne poklapaju; klizi duž ivice.

### 2. Svaki prozor pamti svoj monitor

Svaki prozor pamti monitor na kom je bio, po identitetu tog monitora (proizvođač, model i serijski broj iz podataka samog monitora, EDID), a ne po „monitor 1” ili „monitor 2”. Izvučeš laptop iz stonog monitora i prozori pređu na ekran laptopa; vratiš ga i oni se vrate gde su bili. Zatvoriš program i ponovo ga otvoriš, i on se otvori na istom monitoru, na istom mestu.

### 3. Pomeri na monitor jednim klikom

- Desni klik na naslovnu traku: **Pomeri na › Monitor 2 (Dell, desno)**, sa imenom monitora i malom slikom rasporeda.
- Tastatura: **Neo + Shift + strelica** pomera prozor na monitor u tom smeru; **Neo + Shift + broj** na monitor 1, 2, 3.
- U istom meniju: **Uvek otvaraj ovde** pravi pravilo za taj program (pravilo se vidi i briše u Podešavanja › Monitori).

### 4. Traka: na svakom monitoru ili samo na jednom

Korisnik bira:

- **Jedna Traka** na glavnom monitoru (klasično, mirno).
- **Traka na svakom monitoru**, i svaka pokazuje prozore koji su na tom monitoru (kao Windows 11), ili sve prozore (kao MorphOS i KDE).
- Neo meni i sat mogu da ostanu samo na glavnom monitoru, da ostali monitori budu čisti za rad ili film.

### 5. Profili monitora

Profil čuva ceo raspored: koji monitori su upaljeni, njihovu rezoluciju, osvežavanje i položaj, koji je glavni, gde je Traka i **koji program ide na koji monitor**. Prebacivanje je jedan klik na Traci ili jedan taster.

| Profil (primeri) | Šta radi |
| --- | --- |
| **Rad** | Svi monitori upaljeni; pošta i kalendar levo, uređivač u sredini, Neo DOS desno |
| **Bioskop** | Upaljen samo televizor, na najboljem osvežavanju; Bioskop preko celog ekrana; obaveštenja utišana |
| **Prezentacija** | Ekran laptopa pokazuje beleške, projektor samo prozor sa slajdovima (vidi 7) |
| **Igre** | Jedan monitor na najvećem osvežavanju, ostali zatamnjeni ili ugašeni |

Profil može sam da se ponudi kad se njegovi monitori povežu (utakneš televizor i **Bioskop** se ponudi, ne nameće se). Profili su u korisnikovim podešavanjima, pa ih čuvaju i Hronos snimci i Čuvar kopije.

### 6. Video zid bez posebnog programa

Više monitora, ili projektora, radi kao **jedna velika slika**. Kompozitor crta jednu veliku sliku i svakom ekranu šalje njegov deo.

- Ispravka okvira: računa se i razmak koji prave okviri monitora, pa linija preko celog zida ostaje prava.
- Mogu i različiti ekrani; zid se slaže u Podešavanja › Monitori prevlačenjem ekrana po mreži, onako kako vise na zidu.
- Svaki program može da koristi zid: slika, film, mapa, prezentacija, igra ili sam sto.
- Veći zidovi (više izlaza nego što jedna kartica ima) preko nekoliko NBPOS računara u istom komšiluku (Zadruga, vidi [FEATURES.sr.md](FEATURES.sr.md)), usklađenih preko lokalne mreže. Ovo dolazi posle zida na jednom računaru.

### 7. Prikaži jedan prozor na drugom monitoru

**Prikaži na › Projektor** stavlja živu kopiju jednog prozora na drugi monitor (ili zid) preko celog ekrana, dok ti nastavljaš da radiš na svom ekranu. Korisno za nastavu, prezentacije i film na žurci. Ništa drugo sa tvog stola se tamo ne vidi, pa obaveštenja i ostali prozori ostaju privatni.

### 8. Nagoveštaj za programe (API)

Programi pisani sa Neo Kit-om mogu da zamole, ne mogu da nametnu:

- „otvori me na monitoru gde je pokazivač”, „na glavnom monitoru” ili „na monitoru gde je moj drugi prozor”;
- „prikaži me preko celog ekrana na monitoru N” (Bioskop, Arkada, prezentacije);
- „ja sam program za prezentaciju: beleške ovde, slika na drugom monitoru”;
- „reci mi veličinu i položaj video zida”.

Sistem i korisnikova pravila uvek imaju prednost, pa program ne može da zauzme ekran koji mu korisnik nije dao. Programi iz Linux boksa dobijaju isto ponašanje uobičajenim Wayland i X11 putem, gde to podržavaju.

### 9. Ekrani po monitoru (Amiga ideja)

Sa povlačenjem ekrana ([AMIGA.sr.md](AMIGA.sr.md), A10) svaki monitor ima svoju gomilu ekrana: povučeš program preko celog ekrana nadole na jednom monitoru, a ostali ostaju kakvi su.

## Šta treba napraviti

| Deo | Zavisi od | Procena |
| --- | --- | --- |
| Pronalaženje monitora, čitanje EDID-a, režimi i utikanje u radu | izvorni drajveri ekrana ili Linux domen drajvera (DRM/KMS) | deo posla na grafičkim drajverima |
| Jedan sto preko monitora, različite rezolucije i uvećanja, pokazivač preko ivica | kompozitor (bafer po prozoru) | oko 6 čovek-meseci |
| Prozori pamte monitor, „Pomeri na”, tasteri, pravila | gornje | oko 1,5 čovek-mesec |
| Traka na jednom ili svakom monitoru | Traka | oko 1 čovek-mesec |
| Profili monitora | gornje | oko 1 čovek-mesec |
| Prikaz jednog prozora na drugom monitoru | kompozitor | oko 1 čovek-mesec |
| Video zid na jednom računaru (ispravka okvira, stranica za raspored) | gornje | oko 3 čovek-meseca |
| Video zid preko više računara | Zadruga, mreža | oko 4 čovek-meseca |
| Nagoveštaji u Neo Kit API-ju | Neo Kit | oko 0,5 čovek-meseca |

Ukupno oko **18 čovek-meseci**, povrh grafičkih drajvera. Ovo su procene, ne izmeren rad.

## Gde smo danas

Tvrđava koristi samo jedan ekran, onaj koji podesi pokretač sistema (UEFI GOP ili VBE); izvornog grafičkog drajvera još nema, pa se koristi samo jedan monitor i kad ih je povezano više. Rad na više monitora počinje kad kompozitor dobije bafer po prozoru i kad prvi izvorni drajver ekrana ili drajver iz Linux domena može da nabroji monitore.

Merila: Raspberry Pi 5 ima dva mikro HDMI izlaza (dva 4K monitora na 60 Hz). Kartica Intel Arc B580, kao na referentnoj test mašini ([TEST-MACHINES.sr.md](TEST-MACHINES.sr.md)), ima četiri izlaza, dovoljno za mali video zid sa jednog računara.

## Pošteno poređenje

| Funkcija | Najbliže postojeće |
| --- | --- |
| Prozori pamte svoj monitor | Windows 11 i macOS pamte pri ponovnom povezivanju u nekim slučajevima, ne po identitetu monitora i ne po programu |
| Pravila „Uvek otvaraj ovde” | KDE pravila prozora (skrivena, tehnička), DisplayFusion (plaćen, samo Windows) |
| Profili monitora sa prozorima | DisplayFusion i slični dodatni programi; nijedan sistem to nema ugrađeno |
| Video zid | NVIDIA Mosaic i AMD Eyefinity (samo neke kartice), posebni programi za video zidove |
| Prikaz jednog prozora drugde | prikaz za predavača unutar PowerPoint-a ili Impress-a, deljenje prozora u programima za pozive |

Vidi i [UNIQUE-FEATURES.sr.md](UNIQUE-FEATURES.sr.md).
